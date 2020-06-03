use ctrlc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
