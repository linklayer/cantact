use crate::Error;
use cantact::Channel;
use clap::ArgMatches;

use crate::config::Config;

pub fn cmd(_matches: &ArgMatches) -> Result<(), Error> {
    let _config = Config::read();

    let c1 = Channel {
        bitrate: 500000,
        loopback: false,
        listen_only: false,
        enabled: true,
    };
    let c2 = Channel {
        bitrate: 500000,
        loopback: false,
        listen_only: false,
        enabled: true,
    };

    let config = Config {
        channels: vec![c1, c2],
    };

    config.write().unwrap();

    Ok(())
}
