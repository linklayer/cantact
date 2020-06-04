use crate::Error;
use cantact::{Frame, Interface};
use clap::ArgMatches;
use log::info;

use crate::config::Config;
use crate::helpers;

fn print_frame(f: Frame) {
    let mut s = format!("  ch:{}  {:03X}   [{}]  ", f.channel, f.can_id, f.can_dlc);
    for b in f.data.iter().take(f.can_dlc as usize) {
        s = format!("{}{:02X} ", s, b);
    }
    println!("{}", s)
}

pub fn cmd(matches: &ArgMatches) -> Result<(), Error> {
    let flag = helpers::initialize_ctrlc();
    let mut config = Config::read();

    let ch = helpers::parse_channel(&matches)?;
    match ch {
        None => { /* no channel specified, follow config */ }
        Some(ch) => {
            // channel specified, disable all others
            for n in 0..config.channels.len() {
                if n != ch {
                    config.channels[n].enabled = false;
                }
            }
        }
    }
    info!("config: {:?}", config);

    // initialize the interface
    let mut i = Interface::new()?;
    config.apply_to_interface(&mut i)?;

    // start the device
    info!("starting dump");
    i.start(move |f: Frame| {
        print_frame(f);
    })
    .expect("failed to start device");

    helpers::wait_for_ctrlc(&flag);

    i.stop().expect("failed to stop device");
    Ok(())
}
