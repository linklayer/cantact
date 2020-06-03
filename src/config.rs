use crate::Error;
use cantact::{Channel, Interface};
use app_dirs::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

const APP_INFO: AppInfo = AppInfo{name: "cantact", author: "Linklayer"};
const CFG_FILE: &'static str = "cantact.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "channel")]
    pub channels: Vec<Channel>,
}

impl Config {
    // since config files are not manadatory, this should never fail
    pub fn read() -> Config {
        let dir = match get_app_root(AppDataType::UserConfig, &APP_INFO) {
            Ok(d) => d,
            Err(_) => return Config { channels: vec![] },
        };
        let filename = Path::new("").join(dir).join(CFG_FILE);
        let s = match fs::read_to_string(filename) {
            Ok(s) => s,
            Err(_) => return Config { channels: vec![] },
        };
        toml::from_str(&s).unwrap_or(Config { channels: vec![] })
    }

    pub fn write(&self) -> io::Result<()> {
        let dir = match get_app_root(AppDataType::UserConfig, &APP_INFO) {
            Ok(d) => d,
            Err(_) => panic!("could not determine configuraton directory for this platform"),
        };
        fs::create_dir_all(&dir)?;
        let filename = Path::new("").join(dir).join(CFG_FILE);
        let mut file = File::create(filename)?;
        file.write_all(toml::to_string(&self).unwrap().as_bytes())
    }

    pub fn apply_to_interface(&self, i: &mut Interface) -> Result<(), Error> {
        for (n, ch) in self.channels.iter().enumerate() {
            println!("{}, {:?}", n, ch);
            i.set_bitrate(n, ch.bitrate)?;
        }
        Ok(())
    }
}
