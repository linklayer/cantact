use crate::Error;
use clap::ArgMatches;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const MAX_CHANNELS: usize = 2;

pub fn initialize_ctrlc() -> Arc<AtomicBool> {
    let flag = Arc::new(AtomicBool::new(false));
    let f = flag.clone();

    ctrlc::set_handler(move || {
        f.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    flag
}

pub fn check_ctrlc(f: &Arc<AtomicBool>) -> bool {
    f.load(Ordering::SeqCst)
}

pub fn wait_for_ctrlc(f: &Arc<AtomicBool>) {
    while !check_ctrlc(f) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

pub fn parse_channel(matches: &ArgMatches) -> Result<Option<usize>, Error> {
    if !matches.is_present("channel") {
        return Ok(None);
    }
    let ch_str = matches.value_of("channel").unwrap();
    match ch_str.parse::<usize>() {
        Err(_) => Err(Error::InvalidArgument(String::from(
            "invalid channel value",
        ))),
        Ok(ch) if ch > MAX_CHANNELS => Err(Error::InvalidArgument(String::from(
            "channel value out of range",
        ))),
        Ok(ch) => Ok(Some(ch)),
    }
}
