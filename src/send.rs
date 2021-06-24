use crate::Error;
use cantact::{Frame, Interface};
use clap::ArgMatches;
use log::info;
use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::helpers;

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
    i.start(move |_: Frame| {}).expect("failed to start device");

    let mut count = 0;
    let mut f = Frame::default();
    f.can_dlc = 8;
    loop {
        f.can_id = count % 0x800;
        i.send(f.clone()).unwrap();
        count += 1;
        if count % 1000 == 0 {
            println!("{}", count)
        }
        thread::sleep(Duration::from_millis(10));
        if helpers::check_ctrlc(&flag) {
            break;
        }
    }
    i.stop()?;
    Ok(())
}
