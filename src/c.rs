use crate::{Frame, Interface};

#[repr(C)]
pub struct CFrame {
    channel: u8,
    id: u32,
    dlc: u8,
    data: [u8; 8],
}

#[repr(C)]
pub struct CInterface {
    i: Option<Interface>,
    c_rx_cb: Option<extern "C" fn(*const CFrame)>,
}

#[no_mangle]
pub extern "C" fn cantact_init() -> *mut CInterface {
    Box::into_raw(Box::new(CInterface {
        i: None,
        c_rx_cb: None,
    }))
}

#[no_mangle]
pub extern "C" fn cantact_deinit(ptr: *mut CInterface) -> i32 {
    unsafe {
        Box::from_raw(ptr);
    };
    0
}

#[no_mangle]
pub extern "C" fn cantact_set_rx_callback(
    ptr: *mut CInterface,
    cb: Option<extern "C" fn(*const CFrame)>,
) -> i32 {
    let mut ci = unsafe { &mut *ptr };
    ci.c_rx_cb = cb;
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
    let ci = unsafe { &mut *ptr };

    let cb = ci.c_rx_cb.clone();
    match &mut ci.i {
        Some(i) => i
            .start(move |f: Frame| {
                let cf = CFrame {
                    channel: f.channel,
                    id: f.can_id,
                    dlc: f.can_dlc,
                    data: f.data,
                };
                match cb {
                    None => {}
                    Some(cb) => {
                        //let cf = CFrame{channel: 0, id: 1, dlc: 2, data: [1,2,3,4,5,6,7,8]};
                        cb(&cf);
                        // free the allocated box
                        //unsafe {Box::from_raw(cf);};
                    }
                };
            })
            .expect("failed to start device"),
        None => return -1,
    };
    0
}

#[no_mangle]
pub extern "C" fn cantact_stop(ptr: *mut CInterface) -> i32 {
    let ci = unsafe { &mut *ptr };
    match &mut ci.i {
        Some(i) => i.stop().expect("failed to stop device"),
        None => return -1,
    }
    0
}

#[no_mangle]
pub extern "C" fn cantact_transmit(ptr: *mut CInterface, cf: &CFrame) -> i32 {
    let ci = unsafe { &*ptr };
    let f = Frame {
        can_id: cf.id,
        can_dlc: cf.dlc,
        channel: cf.channel,
        data: cf.data,
    };
    match &ci.i {
        Some(i) => i.send(f).expect("failed to transmit frame"),
        None => return -1,
    };
    0
}

#[no_mangle]
pub extern "C" fn cantact_set_bitrate(ptr: *mut CInterface) -> i32 {
    let ci = unsafe { &*ptr };
    match &ci.i {
        Some(i) => i.set_bitrate(0, 500000).expect("failed to set bitrate"),
        None => return -1,
    }
    0
}

#[no_mangle]
pub extern "C" fn cantact_set_bitrate_user(_ptr: *mut CInterface) -> i32 {
    0
}
