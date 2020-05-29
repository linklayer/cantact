//! Implementation of C/C++ bindings.
//!
//! TODO: put a simple example here.
//!
use crate::{Frame, Interface};

/// A CAN frame in a C representation
#[repr(C)]
pub struct CFrame {
    channel: u8,
    id: u32,
    dlc: u8,
    data: [u8; 8],
    ext: bool,
    fd: bool,
    loopback: bool,
    rtr: bool,
}
impl CFrame {
    fn from_frame(f: Frame) -> CFrame {
        CFrame {
            channel: f.channel,
            id: f.can_id,
            dlc: f.can_dlc,
            data: f.data,
            ext: f.ext,
            fd: f.fd,
            loopback: f.loopback,
            rtr: false,
        }
    }
    fn to_frame(&self) -> Frame {
        Frame {
            can_id: self.id,
            can_dlc: self.dlc,
            channel: self.channel,
            data: self.data,
            ext: self.ext,
            fd: self.fd,
            loopback: self.loopback,
            rtr: self.rtr,
        }
    }
}

/// Interface state. A pointer to this struct is provided when initializing the
/// library. All other functions require a pointer to this struct as the first
/// argument.
#[repr(C)]
pub struct CInterface {
    i: Option<Interface>,
    c_rx_cb: Option<extern "C" fn(*const CFrame)>,
}

/// Create a new CANtact interface, returning a pointer to the interface.
/// This pointer must be provided as the first argument to all other calls in
/// this library.
#[no_mangle]
pub extern "C" fn cantact_init() -> *mut CInterface {
    Box::into_raw(Box::new(CInterface {
        i: None,
        c_rx_cb: None,
    }))
}

/// Clean up a CANtact interface.
/// After calling, the pointer is no longer valid.
#[no_mangle]
pub extern "C" fn cantact_deinit(ptr: *mut CInterface) -> i32 {
    unsafe {
        Box::from_raw(ptr);
    };
    0
}

/// Set the receive callback function. This function will be called when a
/// frame is received.
#[no_mangle]
pub extern "C" fn cantact_set_rx_callback(
    ptr: *mut CInterface,
    cb: Option<extern "C" fn(*const CFrame)>,
) -> i32 {
    let mut ci = unsafe { &mut *ptr };
    ci.c_rx_cb = cb;
    0
}

/// Open the device. This must be called before any interaction with the
/// device (changing settings, starting communication).
#[no_mangle]
pub extern "C" fn cantact_open(ptr: *mut CInterface) -> i32 {
    let i = match Interface::new() {
        Ok(i) => i,
        Err(_) => return -1,
    };
    let ci = unsafe { &mut *ptr };
    ci.i = Some(i);
    0
}

/// Close the device. After closing, no interaction with the device
/// can be performed.
#[no_mangle]
pub extern "C" fn cantact_close(ptr: *mut CInterface) -> i32 {
    let mut ci = unsafe { &mut *ptr };
    ci.i = None;
    0
}

/// Start CAN communication. This will enable all configured CAN channels.
///
/// This function starts a thread which will call the registered callback
/// when a frame is received.
#[no_mangle]
pub extern "C" fn cantact_start(ptr: *mut CInterface) -> i32 {
    let ci = unsafe { &mut *ptr };

    let cb = ci.c_rx_cb.clone();
    match &mut ci.i {
        Some(i) => i
            .start(move |f: Frame| {
                match cb {
                    None => {}
                    Some(cb) => {
                        cb(&CFrame::from_frame(f));
                    }
                };
            })
            .expect("failed to start device"),
        None => return -1,
    };
    0
}

/// Stop CAN communication. This will stop all configured CAN channels.
#[no_mangle]
pub extern "C" fn cantact_stop(ptr: *mut CInterface) -> i32 {
    let ci = unsafe { &mut *ptr };
    match &mut ci.i {
        Some(i) => i.stop().expect("failed to stop device"),
        None => return -1,
    }
    0
}

/// Transmit a frame. Can only be called if the device is running.
#[no_mangle]
pub extern "C" fn cantact_transmit(ptr: *mut CInterface, cf: &CFrame) -> i32 {
    let ci = unsafe { &*ptr };
    match &ci.i {
        Some(i) => i.send(cf.to_frame()).expect("failed to transmit frame"),
        None => return -1,
    };
    0
}

/// TODO
#[no_mangle]
pub extern "C" fn cantact_set_bitrate(ptr: *mut CInterface) -> i32 {
    let ci = unsafe { &*ptr };
    match &ci.i {
        Some(i) => i.set_bitrate(0, 500000).expect("failed to set bitrate"),
        None => return -1,
    }
    0
}

/// TODO
#[no_mangle]
pub extern "C" fn cantact_set_bitrate_user(_ptr: *mut CInterface) -> i32 {
    0
}
