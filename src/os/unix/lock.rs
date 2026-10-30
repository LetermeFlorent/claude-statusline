use super::clock::ticks;
use super::PRIVATE_MODE;
use crate::os::{sleep, Claim};

const MS_PER_SECOND: f64 = 1000.0;
const LOCK_POLL_MS: u32 = 1;

pub struct Lock {
    _file: std::fs::File,
}

pub fn lock(path: &str, wait_ms: u32) -> Claim {
    use std::os::unix::fs::OpenOptionsExt;
    let Ok(file) = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .mode(PRIVATE_MODE)
        .open(path)
    else {
        return Claim::Unavailable;
    };
    let deadline = ticks() + f64::from(wait_ms) / MS_PER_SECOND;
    loop {
        match file.try_lock() {
            Ok(()) => return Claim::Held(Lock { _file: file }),
            Err(std::fs::TryLockError::WouldBlock) if ticks() < deadline => sleep(LOCK_POLL_MS),
            Err(std::fs::TryLockError::WouldBlock) => return Claim::Busy,
            Err(_) => return Claim::Unavailable,
        }
    }
}
