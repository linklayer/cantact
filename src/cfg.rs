use crate::Error;
use clap::ArgMatches;

use crate::config::Config;
use crate::helpers;

pub fn cmd(matches: &ArgMatches) -> Result<(), Error> {
    let mut config = Config::read();

    let ch = match helpers::parse_channel(matches)? {
        None => {
            // if no channel is provided, print the current configuration
            print!("{}", config);
            return Ok(());
        }
        Some(ch) => ch,
    };

    if matches.is_present("disable") {
        config.channels[ch].enabled = false;
    } else {
        config.channels[ch].enabled = true;
    }

    if matches.is_present("loopback") {
        config.channels[ch].loopback = true;
    } else {
        config.channels[ch].loopback = false;
    }

    if matches.is_present("monitor") {
        config.channels[ch].monitor = true;
    } else {
        config.channels[ch].monitor = false;
    }

    if matches.is_present("bitrate") {
        let bitrate = match matches.value_of("bitrate").unwrap().parse::<u32>() {
            Err(_) => {
                return Err(Error::InvalidArgument(String::from(
                    "invalid bitrate value",
                )))
            }
            Ok(b) => b,
        };
        config.channels[ch].bitrate = bitrate;
    }

    config.write().unwrap();

    print!("{}", config);
    Ok(())
}
