use cantact::Error as DevError;
use clap::load_yaml;
use clap::App;

// commands
mod cfg;
mod dump;
mod send;

pub mod config;
pub mod helpers;

#[derive(Debug)]
pub enum Error {
    DeviceError(DevError),
}
impl From<DevError> for Error {
    fn from(de: DevError) -> Error {
        Error::DeviceError(de)
    }
}

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    let result = match matches.subcommand() {
        ("dump", Some(m)) => dump::cmd(m),
        ("send", Some(m)) => send::cmd(m),
        ("cfg", Some(m)) => cfg::cmd(m),
        _ => Ok(()),
    };

    match result {
        Ok(_) => {}
        Err(e) => println!("error: {:?}", e),
    }
}
