use cantact::{Frame, Interface};
use std::thread;
use std::time::Duration;

fn print_frame(f: Frame) {
    let mut s = format!("  ch:{}  {:03X}   [{}]  ", f.channel, f.can_id, f.can_dlc);
    for b in f.data.iter().take(f.can_dlc as usize) {
        s = format!("{}{:02X} ", s, b);
    }
    println!("{}", s)
}

fn main() {
    // initialize the interface
    let mut i = Interface::new().expect("error opening device");
    // configure the CAN channel
    i.set_bitrate(0, 500000).expect("error setting bitrate");

    // start the device
    // provides a closure to be called when a frame is received
    i.start(move |f: Frame| {
        print_frame(f);
    })
    .expect("failed to start device");

    loop {
        thread::sleep(Duration::from_millis(1000));
    }
}
