use candrv::*;

fn print_frame(f: gs_host_frame) {
    print!("  ch:{}  {:03X}   [{}]  ", f.channel, f.can_id, f.can_dlc);
    for b in f.data.iter() {
        print!("{:02X} ", b);
    }
    println!("");
}

fn main() {
    let d = Device::new().expect("failed to open device");
    let bt = gs_device_bittiming {
        prop_seg: 0,
        phase_seg1: 13,
        phase_seg2: 2,
        sjw: 1,
        brp: 6,
    };
    d.set_bit_timing(0, bt).expect("failed to set bit timing");
    d.set_mode(
        0,
        gs_device_mode {
            mode: gs_can_mode::GS_CAN_MODE_START as u32,
            flags: 0,
        },
    )
    .expect("failed to start device");

    loop {
        match d.get_frame() {
            Ok(f) => print_frame(f),
            Err(_) => {}
        }
    }
}
