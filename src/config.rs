use crate::Error;
use cantact::{Channel, Interface};
use dirs;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

// TODO platform dependance
static CFG_FILE: &'static str = ".config/cantact.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "channel")]
    pub channels: Vec<Channel>,
}

impl Config {
    // since config files are not manadatory, this should never fail
    pub fn read() -> Config {
        let home = match dirs::home_dir() {
            Some(h) => h,
            None => return Config { channels: vec![] },
        };
        let filename = Path::new("").join(home).join(CFG_FILE);
        let s = match read_to_string(filename) {
            Ok(s) => s,
            Err(_) => return Config { channels: vec![] },
        };
        toml::from_str(&s).unwrap_or(Config { channels: vec![] })
    }

    pub fn write(&self) -> io::Result<()> {
        let home = dirs::home_dir().expect("could not determine home directory");
        let filename = Path::new("").join(home).join(CFG_FILE);
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
