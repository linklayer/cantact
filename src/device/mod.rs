#![allow(dead_code)]
use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError};
use libc::c_void;
use libusb1_sys::constants::*;
use libusb1_sys::*;
use std::mem;
use std::mem::size_of;
use std::ptr;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

pub mod gsusb;
pub(crate) use gsusb::*;

// CANtact USB VID / PID
const USB_VID: u16 = 0x1d50; //0x606f
const USB_PID: u16 = 0x6070; //0x606f

// number of bulk in transfers
const BULK_IN_TRANSFER_COUNT: usize = 32;
// buffer size for bulk in transfer
const BULK_IN_BUF_SIZE: usize = 32;
// timeout for bulk in transfers
const BULK_IN_TIMEOUT_MS: u32 = 5000;

#[derive(Debug)]
pub(crate) enum Error {
    LibusbError(i32),
    DeviceNotFound,
    TransferAllocFailed,
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
            0 => UsbContext {
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

#[derive(Debug)]
pub(crate) struct Device {
    ctx: Arc<UsbContext>,
    hnd: ptr::NonNull<libusb_device_handle>,
    ctrl_transfer: ptr::NonNull<libusb_transfer>,
    ctrl_buf: Vec<u8>,
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

        match unsafe { libusb_claim_interface(hnd, 0) } {
            0 => {}
            e => return Err(Error::LibusbError(e)),
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

            ctrl_transfer: unsafe { ptr::NonNull::new_unchecked(ctrl_transfer) },
            ctrl_buf: vec![],
            ctrl_transfer_pending: RwLock::from(false),

            out_transfer: unsafe { ptr::NonNull::new_unchecked(ctrl_transfer) },
            out_buf: vec![],
            out_transfer_pending: RwLock::from(false),

            in_transfers: [ptr::null_mut(); BULK_IN_TRANSFER_COUNT],
            in_bufs: in_bufs,

            can_rx_send: send,
            can_rx_recv: recv,
        };

        // start the libusb event thread
        let ctx = d.ctx.clone();
        thread::spawn(move || loop {
            unsafe {
                libusb_handle_events(ctx.as_ptr());
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
                e => return Err(Error::LibusbError(e)),
            };
        }
        Ok(())
    }

    pub(crate) fn stop_transfers(&self) -> Result<(), Error> {
        // cancel all bulk in transfers
        for xfer in self.in_transfers.iter() {
            match unsafe { libusb_cancel_transfer(*xfer) } {
                LIBUSB_SUCCESS => {}
                e => return Err(Error::LibusbError(e)),
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
        let buf = &mut self.ctrl_buf;
        let mut transfer = unsafe { &mut *self.ctrl_transfer.as_ptr() };

        let mut setup = vec![
            request_type,
            request,
            (value & 0xFF) as u8,
            (value >> 8) as u8,
            (index & 0xFF) as u8,
            (index >> 8) as u8,
            (data.len() & 0xFF) as u8,
            (data.len() >> 8) as u8,
        ];
        buf.clear();
        buf.append(&mut setup);
        buf.append(&mut data.to_vec());

        transfer.dev_handle = self.hnd.as_ptr();
        transfer.endpoint = 0; // control EP
        transfer.transfer_type = LIBUSB_TRANSFER_TYPE_CONTROL;
        transfer.timeout = 1000;
        transfer.buffer = buf.as_mut_ptr();
        transfer.length = buf.len() as i32;
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
        let rt = 0b01000001;
        self.fill_control_transfer(rt, req as u8, channel, 0, data);
        *self.ctrl_transfer_pending.write().unwrap() = true;
        match unsafe { libusb_submit_transfer(self.ctrl_transfer.as_ptr()) } {
            LIBUSB_SUCCESS => {}
            e => return Err(Error::LibusbError(e)),
        }

        // wait for transfer to complete
        while *self.ctrl_transfer_pending.read().unwrap() {}

        Ok(())
    }

    fn control_in(&mut self, req: UsbBreq, channel: u16, data: &mut [u8]) -> Result<usize, Error> {
        let rt = 0b11000001;
        self.fill_control_transfer(rt, req as u8, channel, 0, data);
        *self.ctrl_transfer_pending.write().unwrap() = true;
        match unsafe { libusb_submit_transfer(self.ctrl_transfer.as_ptr()) } {
            LIBUSB_SUCCESS => {}
            e => return Err(Error::LibusbError(e)),
        }

        // wait for transfer to complete
        while *self.ctrl_transfer_pending.read().unwrap() {}

        Ok(0)
    }

    pub(crate) fn set_host_format(&mut self, val: u32) -> Result<(), Error> {
        let channel = 0;
        self.control_out(UsbBreq::HostFormat, channel, &val.to_le_bytes())
    }

    pub(crate) fn set_bit_timing(&mut self, channel: u16, timing: BitTiming) -> Result<(), Error> {
        self.control_out(UsbBreq::BitTiming, channel, &timing.to_le_bytes())
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
        let mut buf: [u8; size_of::<DeviceConfig>()] = [0u8; size_of::<DeviceConfig>()];
        self.control_in(UsbBreq::DeviceConfig, channel, &mut buf)?;
        Ok(DeviceConfig::from_le_bytes(&buf))
    }

    pub(crate) fn get_bit_timing_consts(&mut self) -> Result<BitTimingConsts, Error> {
        let channel = 0;
        let mut buf: [u8; size_of::<BitTimingConsts>()] = [0u8; size_of::<BitTimingConsts>()];
        self.control_in(UsbBreq::BitTimingConsts, channel, &mut buf)?;
        Ok(BitTimingConsts::from_le_bytes(&buf))
    }

    pub(crate) fn get_timestamp(&mut self) -> Result<u32, Error> {
        let channel = 0;
        let mut buf: [u8; size_of::<u32>()] = [0u8; size_of::<u32>()];
        self.control_in(UsbBreq::Timestamp, channel, &mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    pub(crate) fn send(&mut self, frame: HostFrame) -> Result<(), Error> {
        self.out_buf.clear();
        self.out_buf.append(&mut frame.to_le_bytes());

        self.fill_bulk_out_transfer(self.out_transfer.as_ptr());
        *self.out_transfer_pending.write().unwrap() = true;

        match unsafe { libusb_submit_transfer(self.out_transfer.as_ptr()) } {
            LIBUSB_SUCCESS => {}
            e => return Err(Error::LibusbError(e)),
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
            Err(e) => panic!(e),
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        self.stop_transfers().unwrap();
        unsafe {
            libusb_release_interface(self.hnd.as_ptr(), 0);
            libusb_close(self.hnd.as_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_frame_transmit() {
        let usb = UsbContext::new();
        let mut d = Device::new(usb).unwrap();

        let bt = BitTiming {
            prop_seg: 0,
            phase_seg1: 13,
            phase_seg2: 2,
            sjw: 1,
            brp: 6,
        };
        d.set_bit_timing(0, bt).unwrap();

        d.set_mode(
            0,
            Mode {
                mode: CanMode::Start as u32,
                flags: 0,
            },
        )
        .unwrap();

        let f = HostFrame {
            echo_id: 1,
            can_id: 0x456,
            can_dlc: 2,
            channel: 0,
            flags: 0,
            reserved: 0,
            data: [0xCA, 0xFE, 0, 0, 0, 0, 0, 0],
        };
        d.send(f).unwrap();
    }
    #[test]
    fn test_frame_receive() {
        let usb = UsbContext::new();
        let mut d = Device::new(usb).unwrap();
        d.start_transfers();

        let bt = BitTiming {
            prop_seg: 0,
            phase_seg1: 13,
            phase_seg2: 2,
            sjw: 1,
            brp: 6,
        };
        d.set_bit_timing(0, bt).unwrap();

        d.set_mode(
            0,
            Mode {
                mode: CanMode::Start as u32,
                flags: 0,
            },
        )
        .unwrap();

        println!("{:X?}", d.recv());

        d.set_mode(
            0,
            Mode {
                mode: CanMode::Reset as u32,
                flags: 0,
            },
        )
        .unwrap();
        d.stop_transfers();
    }
}
