use crate::Error;
use directories::{ProjectDirs};
use cantact::{Channel, Interface};
use log::info;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;


const CFG_FILE: &str = "cantact.toml";
const DEFAULT_CONFIG: Channel = Channel {
    bitrate: 500_000,
    data_bitrate: 500_000,
    loopback: false,
    monitor: false,
    fd: false,
    enabled: true,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "channel")]
    pub channels: Vec<Channel>,
}
impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Channels:")?;
        for (n, ch) in self.channels.iter().enumerate() {
            writeln!(f, "\t{} -> {:?}", n, ch)?;
        }
        Ok(())
    }
}

impl Config {
    pub fn default() -> Config {
        Config {
            channels: vec![DEFAULT_CONFIG, DEFAULT_CONFIG],
        }
    }

    // since config files are not mandatory, this should never fail
    // TODO: Perhaps implement ProjectDirs::from_path check as backwards compat w/ 0.0.1??
    pub fn read() -> Config {
        let dir = ProjectDirs::from("com", "linklayer", "cantact");

        let filename = Path::new(dir.unwrap().config_dir()).join(CFG_FILE);
        let s = match fs::read_to_string(&filename) {
            Ok(s) => s,
            Err(_) => return Config::default(),
        };
        let result = toml::from_str(&s).unwrap_or_else(|_| Config::default());
        info!("read configuration from {:?}", filename);
        result
    }

    pub fn write(&self) -> io::Result<()> {
        let dir = ProjectDirs::from("com", "linklayer", "cantact");
        let filename = Path::new(dir.unwrap().config_dir()).join(CFG_FILE);
        info!("writing configuration to {:?}", filename);

        let mut file = File::create(filename)?;
        file.write_all(toml::to_string(&self).unwrap().as_bytes())
    }

    pub fn apply_to_interface(&self, i: &mut Interface) -> Result<(), Error> {
        for (n, ch) in self.channels.iter().enumerate() {
            if n > (i.channels() - 1) {
                // device doesn't have as many channels as config, ignore the rest
                break;
            }
            i.set_bitrate(n, ch.bitrate)?;
            i.set_enabled(n, ch.enabled)?;
            i.set_loopback(n, ch.loopback)?;
            i.set_monitor(n, ch.monitor)?;
            if i.supports_fd() {
                i.set_fd(n, ch.fd)?;
                i.set_data_bitrate(n, ch.data_bitrate)?;
            }
        }
        Ok(())
    }
}