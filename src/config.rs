use crate::Error;
use app_dirs::*;
use cantact::{Channel, Interface};
use log::info;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

const APP_INFO: AppInfo = AppInfo {
    name: "cantact",
    author: "Linklayer",
};
const CFG_FILE: &'static str = "cantact.toml";
const DEFAULT_CONFIG: Channel = Channel {
    bitrate: 500000,
    loopback: false,
    monitor: false,
    enabled: true,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "channel")]
    pub channels: Vec<Channel>,
}
impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Channels:\n")?;
        for (n, ch) in self.channels.iter().enumerate() {
            write!(f, "\t{} -> {:?}\n", n, ch)?;
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
    pub fn read() -> Config {
        let dir = match get_app_root(AppDataType::UserConfig, &APP_INFO) {
            Ok(d) => d,
            Err(_) => return Config::default(),
        };
        let filename = Path::new("").join(dir).join(CFG_FILE);
        let s = match fs::read_to_string(&filename) {
            Ok(s) => s,
            Err(_) => return Config::default(),
        };
        let result = toml::from_str(&s).unwrap_or(Config::default());
        info!("read configuration from {:?}", filename);
        result
    }

    pub fn write(&self) -> io::Result<()> {
        let dir = match get_app_root(AppDataType::UserConfig, &APP_INFO) {
            Ok(d) => d,
            Err(_) => panic!("could not determine configuration directory for this platform"),
        };
        fs::create_dir_all(&dir)?;
        let filename = Path::new("").join(dir).join(CFG_FILE);
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
        }
        Ok(())
    }
}
