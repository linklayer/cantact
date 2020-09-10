//! Implementation of C/C++ bindings.
//!
//! All functions are unsafe since they dereference a context pointer
//! provided from C.
//!
//! TODO: put a simple example here.
//!

#![allow(clippy::missing_safety_doc)]

use crate::{Frame, Interface};

/// A CAN frame in a C representation
#[repr(C)]
pub struct CFrame {
    channel: u8,
    id: u32,
    dlc: u8,
    data: [u8; 64],
    // these types are boolean flags, but C FFI hates bools
    // use u8s instead: 1 = true, 0 = false
    ext: u8,
    fd: u8,
    brs: u8,
    esi: u8,
    loopback: u8,
    rtr: u8,
    err: u8,
}
impl CFrame {
    fn from_frame(f: Frame) -> CFrame {
        CFrame {
            channel: f.channel,
            id: f.can_id,
            dlc: f.can_dlc,
            data: f.data_as_array(),
            ext: if f.ext { 1 } else { 0 },
            fd: if f.fd { 1 } else { 0 },
            brs: if f.brs { 1 } else { 0 },
            esi: if f.esi { 1 } else { 0 },
            loopback: if f.loopback { 1 } else { 0 },
            rtr: if f.rtr { 1 } else { 0 },
            err: if f.err { 1 } else { 0 },
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
///
/// If this function fails, it returns a null pointer (0).
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
pub unsafe extern "C" fn cantact_deinit(ptr: *mut CInterface) -> i32 {
    Box::from_raw(ptr);
    0
}

/// Set the receive callback function. This function will be called when a
/// frame is received.
#[no_mangle]
pub unsafe extern "C" fn cantact_set_rx_callback(
    ptr: *mut CInterface,
    cb: Option<extern "C" fn(*const CFrame)>,
) -> i32 {
    let mut ci = &mut *ptr;
    ci.c_rx_cb = cb;
    0
}

/// Open the device. This must be called before any interaction with the
/// device (changing settings, starting communication).
#[no_mangle]
pub unsafe extern "C" fn cantact_open(ptr: *mut CInterface) -> i32 {
    let i = match Interface::new() {
        Ok(i) => i,
        Err(_) => return -1,
    };
    let ci = &mut *ptr;
    ci.i = Some(i);
    0
}

/// Close the device. After closing, no interaction with the device
/// can be performed.
#[no_mangle]
pub unsafe extern "C" fn cantact_close(ptr: *mut CInterface) -> i32 {
    let mut ci = &mut *ptr;
    ci.i = None;
    0
}

/// Start CAN communication. This will enable all configured CAN channels.
///
/// This function starts a thread which will call the registered callback
/// when a frame is received.
#[no_mangle]
pub unsafe extern "C" fn cantact_start(ptr: *mut CInterface) -> i32 {
    let ci = &mut *ptr;

    let cb = ci.c_rx_cb;
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
pub unsafe extern "C" fn cantact_stop(ptr: *mut CInterface) -> i32 {
    let ci = &mut *ptr;
    match &mut ci.i {
        Some(i) => i.stop().expect("failed to stop device"),
        None => return -1,
    }
    0
}

/// Transmit a frame. Can only be called if the device is running.
#[no_mangle]
pub unsafe extern "C" fn cantact_transmit(ptr: *mut CInterface, cf: CFrame) -> i32 {
    let ci = &mut *ptr;
    let f = Frame {
        channel: 0, //cf.channel,
        can_id: cf.id,
        can_dlc: cf.dlc,
        data: cf.data.to_vec(),
        ext: cf.ext > 0,
        fd: cf.fd > 0,
        brs: cf.brs > 0,
        esi: cf.esi > 0,
        loopback: false,
        rtr: cf.rtr > 0,
        err: cf.err > 0,
        timestamp: None,
    };
    match &mut ci.i {
        Some(i) => i.send(f).expect("failed to transmit frame"),
        None => return -1,
    };
    0
}

/// Sets the bitrate for a chanel to the given value in bits per second.
#[no_mangle]
pub unsafe extern "C" fn cantact_set_bitrate(
    ptr: *mut CInterface,
    channel: u8,
    bitrate: u32,
) -> i32 {
    let ci = &mut *ptr;
    match &mut ci.i {
        Some(i) => i
            .set_bitrate(channel as usize, bitrate)
            .expect("failed to set bitrate"),
        None => return -1,
    }
    0
}

/// Enable or disable a channel.
#[no_mangle]
pub unsafe extern "C" fn cantact_set_enabled(
    ptr: *mut CInterface,
    channel: u8,
    enabled: u8,
) -> i32 {
    let ci = &mut *ptr;
    match &mut ci.i {
        Some(i) => i
            .set_enabled(channel as usize, enabled > 0)
            .expect("failed to enable channel"),
        None => return -1,
    }
    0
}

/// Enable or disable bus monitoring mode for a channel. When enabled, channel
/// will not transmit frames or acknoweldgements.
#[no_mangle]
pub unsafe extern "C" fn cantact_set_monitor(
    ptr: *mut CInterface,
    channel: u8,
    enabled: u8,
) -> i32 {
    let ci = &mut *ptr;
    match &mut ci.i {
        Some(i) => i
            .set_monitor(channel as usize, enabled > 0)
            .expect("failed to set monitoring mode"),
        None => return -1,
    }
    0
}

/// Enable or disable hardware loopback for a channel. This will cause sent
/// frames to be received. This mode is mostly intended for device testing.
#[no_mangle]
pub unsafe extern "C" fn cantact_set_hw_loopback(
    ptr: *mut CInterface,
    channel: u8,
    enabled: u8,
) -> i32 {
    let ci = &mut *ptr;
    match &mut ci.i {
        Some(i) => i
            .set_loopback(channel as usize, enabled > 0)
            .expect("failed to enable channel"),
        None => return -1,
    }
    0
}

/// Get the number of CAN channels the device has.
///
/// Returns the number of channels or a negative error code on failure.
#[no_mangle]
pub unsafe extern "C" fn cantact_get_channel_count(ptr: *mut CInterface) -> i32 {
    let ci = &mut *ptr;
    match &mut ci.i {
        Some(i) => i.channels() as i32,
        None => -1,
    }
}
