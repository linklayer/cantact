use crate::{Interface, Frame, Error};

#[repr(C)]
pub struct CInterface {
    i: Option<Interface>,
}

#[no_mangle]
pub extern "C" fn cantact_init() -> *mut CInterface {
    Box::into_raw(Box::new(CInterface{i: None}))
}

#[no_mangle]
pub extern "C" fn cantact_deinit(ptr: *mut CInterface) -> i32 {
    unsafe { Box::from_raw(ptr); }; 
    0
}

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

#[no_mangle]
pub extern "C" fn cantact_close(ptr: *mut CInterface) -> i32 {
    let mut ci = unsafe { &mut *ptr };
    ci.i = None;
    0
}

#[no_mangle]
pub extern "C" fn cantact_start(ptr: *mut CInterface) -> i32 {
    let ci = unsafe { &*ptr };
    match &ci.i {
        Some(i) => i.start(0),
        None => return -1,
    }
    0
}

#[no_mangle]
pub extern "C" fn cantact_stop(ptr: *mut CInterface) -> i32 {
    let ci = unsafe { &*ptr };
    match &ci.i {
        Some(i) => i.stop(0),
        None => return -1,
    }
    0
}

#[no_mangle]
pub extern "C" fn cantact_transmit(ptr: *mut CInterface, id: u32) -> i32 {
    let ci = unsafe { &*ptr };
    let f = Frame {
        can_id: id,
        can_dlc: 8,
        channel: 0,
        data: [0xde, 0xad, 0xbe, 0xef, 0,0,0,0],
    };
    match &ci.i {
        Some(i) => i.send(f).unwrap(),
        None => return -1,
    };
    0
}

#[no_mangle]
pub extern "C" fn cantact_set_bitrate(ptr: *mut CInterface) -> i32 {
    let ci = unsafe { &*ptr };
    match &ci.i {
        Some(i) => i.set_bitrate(0, 500000),
        None => return -1,
    }
    0
}

#[no_mangle]
pub extern "C" fn cantact_set_bitrate_user(ptr: *mut CInterface) -> i32 {
    0
}
