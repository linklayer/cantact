#![allow(dead_code)]
use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError};
use libc::c_void;
use libusb1_sys::constants::*;
use libusb1_sys::*;
use std::mem;
use std::mem::size_of;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

pub mod gsusb;
pub(crate) use gsusb::*;

// CANtact USB VID / PID
const USB_VID: u16 = 0x1d50;
const USB_PID: u16 = 0x606f;

// buffer size for control in/out transfers
const CTRL_BUF_SIZE: usize = 64;
// number of bulk in transfers
const BULK_IN_TRANSFER_COUNT: usize = 32;
// buffer size for bulk in transfer
const BULK_IN_BUF_SIZE: usize = 76;
// timeout for bulk in transfers
const BULK_IN_TIMEOUT_MS: u32 = 5000;

#[derive(Debug)]
pub enum Error {
    LibusbError(&'static str, i32),
    DeviceNotFound,
    TransferAllocFailed,
    InvalidControlResponse,
}

#[derive(Debug)]
pub(crate) struct UsbContext {
    ctx: *mut libusb_context,
}

unsafe impl Send for UsbContext {}
unsafe impl Sync for UsbContext {}

impl UsbContext {
    pub(crate) fn new() -> UsbContext {
        let mut context = mem::MaybeUninit::<*mut libusb_context>::uninit();
        match unsafe { libusb_init(context.as_mut_ptr()) } {
            LIBUSB_SUCCESS => UsbContext {
                ctx: unsafe { context.assume_init() },
            },
            _ => panic!("could not initialize libusb context"),
        }
    }
    fn as_ptr(&self) -> *mut libusb_context {
        self.ctx
    }
}
impl Drop for UsbContext {
    fn drop(&mut self) {
        unsafe { libusb_exit(self.ctx) }
    }
}

pub(crate) struct Device {
    ctx: Arc<UsbContext>,
    hnd: ptr::NonNull<libusb_device_handle>,
    running: Arc<AtomicBool>,

    ctrl_transfer: ptr::NonNull<libusb_transfer>,
    ctrl_buf: [u8; CTRL_BUF_SIZE],
    ctrl_transfer_pending: RwLock<bool>,

    out_transfer: ptr::NonNull<libusb_transfer>,
    out_buf: Vec<u8>,
    out_transfer_pending: RwLock<bool>,

    in_transfers: [*mut libusb_transfer; BULK_IN_TRANSFER_COUNT],
    in_bufs: [[u8; BULK_IN_BUF_SIZE]; BULK_IN_TRANSFER_COUNT],

    can_rx_send: Sender<HostFrame>,
    pub can_rx_recv: Receiver<HostFrame>,
}

extern "system" fn ctrl_cb(xfer: *mut libusb_transfer) {
    let dev_ptr = unsafe { (*xfer).user_data as *mut Device };
    let dev = unsafe { &mut *dev_ptr };

    let _status = unsafe { (*xfer).status };

    *dev.ctrl_transfer_pending.write().unwrap() = false;
}
extern "system" fn bulk_out_cb(xfer: *mut libusb_transfer) {
    let dev_ptr = unsafe { (*xfer).user_data as *mut Device };
    let dev = unsafe { &mut *dev_ptr };
    let _status = unsafe { (*xfer).status };

    *dev.out_transfer_pending.write().unwrap() = false;
}

extern "system" fn bulk_in_cb(xfer: *mut libusb_transfer) {
    let dev_ptr = unsafe { (*xfer).user_data as *mut Device };
    let dev = unsafe { &mut *dev_ptr };
    let status = unsafe { (*xfer).status };

    if status == LIBUSB_TRANSFER_COMPLETED {
        let frame_data = unsafe { std::slice::from_raw_parts((*xfer).buffer, BULK_IN_BUF_SIZE) };
        let f = HostFrame::from_le_bytes(frame_data);
        dev.can_rx_send.send(f).unwrap();
    }
    if status != LIBUSB_TRANSFER_CANCELLED {
        // resubmit the transfer unless it was cancelled
        unsafe {
            libusb_submit_transfer(xfer);
        }
    }
}

impl Device {
    pub(crate) fn new(ctx: UsbContext) -> Result<Device, Error> {
        let hnd = unsafe { libusb_open_device_with_vid_pid(ctx.as_ptr(), USB_VID, USB_PID) };
        if hnd.is_null() {
            return Err(Error::DeviceNotFound);
        }

        match unsafe { libusb_detach_kernel_driver(hnd, 0) } {
            LIBUSB_SUCCESS => {}
            LIBUSB_ERROR_NOT_FOUND => { /* device already disconnected */ }
            LIBUSB_ERROR_NOT_SUPPORTED => { /* can't detach on this system (not linux) */ }
            e => return Err(Error::LibusbError("libusb_detach_kernel_driver", e)),
        }

        match unsafe { libusb_claim_interface(hnd, 0) } {
            LIBUSB_SUCCESS => {}
            e => return Err(Error::LibusbError("libusb_claim_interface", e)),
        }

        let ctrl_transfer = unsafe { libusb_alloc_transfer(0) };
        if ctrl_transfer.is_null() {
            return Err(Error::TransferAllocFailed);
        }

        let in_bufs: [[u8; BULK_IN_BUF_SIZE]; BULK_IN_TRANSFER_COUNT] =
            [[0u8; BULK_IN_BUF_SIZE]; BULK_IN_TRANSFER_COUNT];

        let (send, recv) = unbounded();

        let d = Device {
            ctx: Arc::new(ctx),
            hnd: unsafe { ptr::NonNull::new_unchecked(hnd) },
            running: Arc::new(AtomicBool::new(true)),

            ctrl_transfer: unsafe { ptr::NonNull::new_unchecked(ctrl_transfer) },
            ctrl_buf: [0u8; CTRL_BUF_SIZE],
            ctrl_transfer_pending: RwLock::from(false),

            out_transfer: unsafe { ptr::NonNull::new_unchecked(ctrl_transfer) },
            out_buf: vec![],
            out_transfer_pending: RwLock::from(false),

            in_transfers: [ptr::null_mut(); BULK_IN_TRANSFER_COUNT],
            in_bufs,

            can_rx_send: send,
            can_rx_recv: recv,
        };

        // start the libusb event thread
        let ctx = d.ctx.clone();
        let running = d.running.clone();
        thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                unsafe {
                    libusb_handle_events(ctx.as_ptr());
                }
            }
        });

        Ok(d)
    }

    pub(crate) fn start_transfers(&mut self) -> Result<(), Error> {
        // create the in transfers, fill the transfers, and submit them
        for i in 0..BULK_IN_TRANSFER_COUNT {
            let xfer = unsafe { libusb_alloc_transfer(0) };
            if xfer.is_null() {
                return Err(Error::TransferAllocFailed);
            }
            self.in_transfers[i] = xfer;
            self.fill_bulk_in_transfer(i);

            match unsafe { libusb_submit_transfer(self.in_transfers[i]) } {
                LIBUSB_SUCCESS => {}
                e => {
                    return Err(Error::LibusbError(
                        "start_transfers: libusb_submit_transfer",
                        e,
                    ))
                }
            };
        }
        Ok(())
    }

    pub(crate) fn stop_transfers(&self) -> Result<(), Error> {
        // cancel all bulk in transfers
        for xfer in self.in_transfers.iter() {
            if xfer.is_null() {
                // ignore null transfers
                continue;
            }
            match unsafe { libusb_cancel_transfer(*xfer) } {
                LIBUSB_SUCCESS => {}
                LIBUSB_ERROR_NOT_FOUND => { /* already destroyed */ }
                e => return Err(Error::LibusbError("libusb_cancel_transfer", e)),
            }
        }
        Ok(())
    }

    fn fill_control_transfer(
        &mut self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        data: &[u8],
    ) {
        let mut transfer = unsafe { &mut *self.ctrl_transfer.as_ptr() };

        // clear buffer
        self.ctrl_buf = [0u8; CTRL_BUF_SIZE];
        // setup packet
        self.ctrl_buf[0] = request_type; // bmRequestType
        self.ctrl_buf[1] = request; // bRequest
        self.ctrl_buf[2] = (value & 0xFF) as u8; // wValue
        self.ctrl_buf[3] = (value >> 8) as u8;
        self.ctrl_buf[4] = (index & 0xFF) as u8; // wIndex
        self.ctrl_buf[5] = (index >> 8) as u8;
        self.ctrl_buf[6] = (data.len() & 0xFF) as u8; // wLength
        self.ctrl_buf[7] = (data.len() >> 8) as u8;

        // copy control out data
        self.ctrl_buf[8..(data.len() + 8)].clone_from_slice(&data[..]);

        transfer.dev_handle = self.hnd.as_ptr();
        transfer.endpoint = 0;
        transfer.transfer_type = LIBUSB_TRANSFER_TYPE_CONTROL;
        transfer.timeout = 1000;
        transfer.buffer = self.ctrl_buf.as_mut_ptr();
        transfer.length = self.ctrl_buf.len() as i32;
        transfer.callback = ctrl_cb;
        transfer.user_data = self as *mut _ as *mut c_void;
    }

    fn fill_bulk_out_transfer(&mut self, transfer: *mut libusb_transfer) {
        let mut transfer = unsafe { &mut *transfer };
        let buf = &mut self.out_buf;

        transfer.dev_handle = self.hnd.as_ptr();
        transfer.endpoint = 0x02; // bulk out ep
        transfer.transfer_type = LIBUSB_TRANSFER_TYPE_BULK;
        transfer.timeout = 1000;
        transfer.buffer = buf.as_mut_ptr();
        transfer.length = buf.len() as i32;
        transfer.callback = bulk_out_cb;
        transfer.user_data = self as *mut _ as *mut c_void;
    }

    fn fill_bulk_in_transfer(&mut self, idx: usize) {
        let mut transfer = unsafe { &mut *self.in_transfers[idx] };
        let buf = &mut self.in_bufs[idx];

        transfer.dev_handle = self.hnd.as_ptr();
        transfer.endpoint = 0x81; // bulk in ep
        transfer.transfer_type = LIBUSB_TRANSFER_TYPE_BULK;
        transfer.timeout = BULK_IN_TIMEOUT_MS;
        transfer.buffer = buf.as_mut_ptr();
        transfer.length = buf.len() as i32;
        transfer.callback = bulk_in_cb;
        transfer.user_data = self as *mut _ as *mut c_void;
    }

    fn control_out(&mut self, req: UsbBreq, channel: u16, data: &[u8]) -> Result<(), Error> {
        // bmRequestType: direction = out, type = vendor, recipient = interface
        let rt = 0b0100_0001;
        self.fill_control_transfer(rt, req as u8, channel, 0, data);
        *self.ctrl_transfer_pending.write().unwrap() = true;
        match unsafe { libusb_submit_transfer(self.ctrl_transfer.as_ptr()) } {
            LIBUSB_SUCCESS => {}
            e => return Err(Error::LibusbError("control_out: libusb_submit_transfer", e)),
        }

        // wait for transfer to complete
        while *self.ctrl_transfer_pending.read().unwrap() {}

        Ok(())
    }

    fn control_in(&mut self, req: UsbBreq, channel: u16, len: usize) -> Result<Vec<u8>, Error> {
        // bmRequestType: direction = in, type = vendor, recipient = interface
        let rt = 0b1100_0001;
        self.fill_control_transfer(rt, req as u8, channel, 0, &vec![0u8; len].as_slice());
        *self.ctrl_transfer_pending.write().unwrap() = true;
        match unsafe { libusb_submit_transfer(self.ctrl_transfer.as_ptr()) } {
            LIBUSB_SUCCESS => {}
            e => return Err(Error::LibusbError("control_in: libusb_submit_transfer", e)),
        }

        // wait for transfer to complete
        while *self.ctrl_transfer_pending.read().unwrap() {}
        let xfer_len = unsafe { (*self.ctrl_transfer.as_ptr()).actual_length } as usize;
        if xfer_len < len {
            // we didn't get the full struct we asked for
            return Err(Error::InvalidControlResponse);
        }

        Ok(self.ctrl_buf[8..8 + xfer_len].to_vec())
    }

    pub(crate) fn set_host_format(&mut self, val: u32) -> Result<(), Error> {
        let channel = 0;
        self.control_out(UsbBreq::HostFormat, channel, &val.to_le_bytes())
    }

    pub(crate) fn set_bit_timing(&mut self, channel: u16, timing: BitTiming) -> Result<(), Error> {
        self.control_out(UsbBreq::BitTiming, channel, &timing.to_le_bytes())
    }

    pub(crate) fn set_data_bit_timing(
        &mut self,
        channel: u16,
        timing: BitTiming,
    ) -> Result<(), Error> {
        self.control_out(UsbBreq::DataBitTiming, channel, &timing.to_le_bytes())
    }

    pub(crate) fn set_mode(&mut self, channel: u16, device_mode: Mode) -> Result<(), Error> {
        self.control_out(UsbBreq::Mode, channel, &device_mode.to_le_bytes())
    }

    pub(crate) fn set_identify(&mut self, val: u32) -> Result<(), Error> {
        let channel = 0;
        self.control_out(UsbBreq::Identify, channel, &val.to_le_bytes())
    }

    pub(crate) fn set_berr(&mut self, val: u32) -> Result<(), Error> {
        // TODO
        let channel = 0;
        self.control_out(UsbBreq::Berr, channel, &val.to_le_bytes())
    }

    pub(crate) fn get_device_config(&mut self) -> Result<DeviceConfig, Error> {
        let channel = 0;
        let data = self.control_in(UsbBreq::DeviceConfig, channel, size_of::<DeviceConfig>())?;
        Ok(DeviceConfig::from_le_bytes(&data))
    }

    pub(crate) fn get_bit_timing_consts(&mut self) -> Result<BitTimingConsts, Error> {
        let channel = 0;
        let data = self.control_in(
            UsbBreq::BitTimingConsts,
            channel,
            size_of::<BitTimingConsts>(),
        )?;
        Ok(BitTimingConsts::from_le_bytes(&data))
    }

    pub(crate) fn get_timestamp(&mut self) -> Result<u32, Error> {
        let channel = 0;
        let data = self.control_in(UsbBreq::Timestamp, channel, size_of::<u32>())?;
        let bytes = [data[0], data[1], data[2], data[3]];
        Ok(u32::from_le_bytes(bytes))
    }

    pub(crate) fn send(&mut self, frame: HostFrame) -> Result<(), Error> {
        self.out_buf.clear();
        self.out_buf.append(&mut frame.to_le_bytes());

        self.fill_bulk_out_transfer(self.out_transfer.as_ptr());
        *self.out_transfer_pending.write().unwrap() = true;

        match unsafe { libusb_submit_transfer(self.out_transfer.as_ptr()) } {
            LIBUSB_SUCCESS => {}
            e => return Err(Error::LibusbError("send: libusb_submit_transfer", e)),
        }

        // wait for transfer to complete
        while *self.out_transfer_pending.read().unwrap() {}

        Ok(())
    }

    pub(crate) fn try_recv(&self) -> Option<HostFrame> {
        match self.can_rx_recv.try_recv() {
            Ok(f) => Some(f),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => None,
        }
    }
    pub(crate) fn recv(&self) -> HostFrame {
        match self.can_rx_recv.recv() {
            Ok(f) => f,
            Err(e) => panic!("{}", e),
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        // stop the threau
        self.running.store(false, Ordering::SeqCst);

        self.stop_transfers().unwrap();
        unsafe {
            libusb_release_interface(self.hnd.as_ptr(), 0);
            libusb_close(self.hnd.as_ptr());
        }
    }
}
