use cantact::{Frame, Interface};
use std::thread;
use std::time;

fn print_frame(f: Frame) {
    print!("  ch:{}  {:03X}   [{}]  ", f.channel, f.can_id, f.can_dlc);
    for b in f.data.iter().take(f.can_dlc as usize) {
        print!("{:02X} ", b);
    }
    println!("");
}

fn main() {
    let mut i = Interface::new().expect("error opening device");
    i.set_bitrate(0, 500000).expect("error setting bitrate");
    //i.set_rx_callback(Some(print_frame)).expect("error setting rx callback");
    i.start(|f: Frame| {
        print_frame(f);
    });

    loop {
        let f = Frame {
            can_id: 0x123,
            can_dlc: 8,
            data: [1, 2, 3, 4, 5, 6, 7, 8],
            channel: 0,
        };
        thread::sleep(time::Duration::from_millis(1000));
        i.send(f).unwrap();
        println!("tx");
    }
}
