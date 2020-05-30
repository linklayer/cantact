use cantact::{Frame, Interface};
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::prelude::*;

fn print_frame(file: &mut File, f: Frame) {
    let mut s = format!("  ch:{}  {:03X}   [{}]  ", f.channel, f.can_id, f.can_dlc);
    for b in f.data.iter().take(f.can_dlc as usize) {
        s = format!("{}{:02X} ", s, b);
    }
    s = format!("{}\n", s);
    file.write_all(s.as_bytes()).unwrap();
    
}

fn main() {
    // initialize the interface
    let mut i = Interface::new().expect("error opening device");
    // configure the CAN channel
    i.set_bitrate(0, 500000).expect("error setting bitrate");

    let mut file = File::create("log.txt").expect("error opening output file");

    // start the device
    // provides a closure to be called when a frame is received
    i.start(move|f: Frame| {
        print_frame(&mut file, f);
    })
    .expect("failed to start device");

    loop {
        thread::sleep(Duration::from_millis(1000));
    }
}
