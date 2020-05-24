use cantact::{Interface, Frame};

fn print_frame(f: Frame) {
    print!("  ch:{}  {:03X}   [{}]  ", f.channel, f.can_id, f.can_dlc);
    for b in f.data.iter().take(f.can_dlc as usize) {
        print!("{:02X} ", b);
    }
    println!("");
}

fn main() {
    let i = Interface::new();
    i.start(0);
    i.set_bitrate(0, 500000);
    loop {
        match i.recv() {
            Some(f) => print_frame(f),
            None => {}
        }
    }
}
