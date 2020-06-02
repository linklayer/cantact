use cantact::{Frame, Interface};
use std::thread;
use std::time::Duration;

fn main() {
    // initialize the interface
    let mut i = Interface::new().expect("error opening device");
    // configure the CAN channel
    i.set_bitrate(0, 500000).expect("error setting bitrate");

    // start the device
    i.start(|_: Frame| {}).expect("failed to start device");

    let mut count = 0;
    let mut f = Frame::default();
    f.can_dlc = 8;
    loop {
        f.can_id = count % 0x800;
        i.send(f.clone()).unwrap();
        count = count + 1;
        if count % 1000 == 0 {
            println!("{}", count)
        }
        thread::sleep(Duration::from_millis(0));
    }
}
