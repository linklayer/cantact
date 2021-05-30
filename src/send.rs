use crate::Error;
use cantact::{Frame, Interface};
use clap::ArgMatches;
use std::thread;
use std::time::Duration;
use std::convert::TryInto;

use crate::helpers;

pub fn cmd(matches: &ArgMatches) -> Result<(), Error> {
    let flag = helpers::initialize_ctrlc();

    // initialize the interface
    let mut i = Interface::new().expect("error opening device");
    // configure the CAN channel
    // TODO: Why is 500_000 fixed in here, shouldn't it be based on config?
    i.set_bitrate(0, 500_000).expect("error setting bitrate");
    // start the device
    i.start(|_: Frame| {}).expect("failed to start device");

    let mut count = 0;
    let mut f = Frame::default();
    let cli_data = &matches.args.get_key_value("data").unwrap().1.vals[0];
    let cli_data_slice = cli_data.to_str().unwrap().as_bytes().try_into().expect("Ooops");
    dbg!(&cli_data_slice);

    f.can_dlc = 8;
    loop {
        f.can_id = count % 0x800;
        f.data = cli_data_slice;
        i.send(f.clone())?;
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
