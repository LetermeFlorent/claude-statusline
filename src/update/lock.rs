use crate::messages;
use crate::os::{self, Claim};
use crate::update_watch::MARKER;
use alloc::string::String;

const LOCK_NAME: &str = "update.lock";
const SECOND_UPDATE_WAIT_MS: u32 = 2000;
const REFRESH_WAIT_MS: u32 = 0;

pub fn take(dir: &str) -> Result<Option<os::Lock>, String> {
    if !os::mkdirs(dir) {
        return Err(messages::STATE_DIR_FAILED.with(&[dir]));
    }
    match os::lock(&os::join(dir, &[LOCK_NAME]), SECOND_UPDATE_WAIT_MS) {
        Claim::Held(lock) => Ok(Some(lock)),
        Claim::Busy => Ok(None),
        Claim::Unavailable => Err(messages::STATE_DIR_FAILED.with(&[dir])),
    }
}

pub fn running(dir: &str) -> bool {
    let marker = os::join(dir, &[MARKER]);
    if !os::exists(&marker) {
        return false;
    }
    match os::lock(&os::join(dir, &[LOCK_NAME]), REFRESH_WAIT_MS) {
        Claim::Held(_free) => {
            os::remove(&marker);
            false
        }
        Claim::Busy => true,
        Claim::Unavailable => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::Sandbox;
    use std::time::{Duration, Instant, SystemTime};

    const DAY: Duration = Duration::from_secs(86_400);
    const SECOND_UPDATE_BOUND: Duration = Duration::from_secs(5);
    const REFRESH_BOUND: Duration = Duration::from_millis(500);

    fn backdate(path: &str) {
        let file = std::fs::File::options().write(true).open(path).unwrap();
        file.set_modified(SystemTime::now() - DAY).unwrap();
    }

    fn state(sandbox: &Sandbox) -> (String, String) {
        let dir = sandbox.path("state");
        std::fs::create_dir_all(&dir).unwrap();
        let marker = os::join(&dir, &[MARKER]);
        std::fs::write(&marker, b"v99.0.0").unwrap();
        (dir, marker)
    }

    #[test]
    fn a_state_folder_that_cannot_be_created_is_an_error() {
        let sandbox = Sandbox::new("blocked");
        let blocker = sandbox.0.join("file");
        std::fs::write(&blocker, b"").unwrap();
        let dir = blocker.join("state").to_string_lossy().into_owned();
        let taken = take(&dir).map(|held| held.is_some());
        assert_eq!(taken, Err(messages::STATE_DIR_FAILED.with(&[&dir])));
    }

    #[test]
    fn a_lock_that_cannot_be_opened_stops_the_update_but_not_the_bar() {
        let sandbox = Sandbox::new("unusable-lock");
        let (dir, marker) = state(&sandbox);
        std::fs::create_dir(os::join(&dir, &[LOCK_NAME])).unwrap();
        assert!(!running(&dir));
        assert!(os::exists(&marker));
        let taken = take(&dir).map(|held| held.is_some());
        assert_eq!(taken, Err(messages::STATE_DIR_FAILED.with(&[&dir])));
    }

    #[test]
    fn a_live_update_is_honoured_past_any_age() {
        let sandbox = Sandbox::new("live-lock");
        let (dir, marker) = state(&sandbox);
        let held = take(&dir).unwrap();
        assert!(held.is_some());
        backdate(&marker);
        backdate(&os::join(&dir, &[LOCK_NAME]));
        assert!(running(&dir));
        assert!(take(&dir).unwrap().is_none());
        assert!(os::exists(&marker));
    }

    #[test]
    fn a_lock_left_by_a_dead_update_is_taken_over() {
        let sandbox = Sandbox::new("dead-lock");
        let (dir, marker) = state(&sandbox);
        std::fs::write(os::join(&dir, &[LOCK_NAME]), b"").unwrap();
        assert!(!running(&dir));
        assert!(!os::exists(&marker));
        assert!(take(&dir).unwrap().is_some());
    }

    #[test]
    fn a_second_update_gives_up_within_a_bounded_time() {
        let sandbox = Sandbox::new("second-update");
        let (dir, _) = state(&sandbox);
        let held = take(&dir).unwrap();
        assert!(held.is_some());
        let start = Instant::now();
        assert!(take(&dir).unwrap().is_none());
        assert!(start.elapsed() < SECOND_UPDATE_BOUND);
    }

    #[test]
    fn a_refresh_never_waits_on_the_update() {
        let sandbox = Sandbox::new("refresh");
        let (dir, _) = state(&sandbox);
        let held = take(&dir).unwrap();
        assert!(held.is_some());
        let start = Instant::now();
        assert!(running(&dir));
        assert!(start.elapsed() < REFRESH_BOUND);
    }
}
