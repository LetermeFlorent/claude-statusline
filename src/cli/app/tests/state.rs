use crate::app::persist::{read_state, save_state_at, CHANGES_SUFFIX};
use crate::sandbox::Sandbox;
use crate::state::State;
use std::os::windows::fs::OpenOptionsExt;
use std::time::Duration;

const SHARE_WRITE_AND_DELETE: u32 = 6;

fn hold(path: &str, held_ms: u64) -> std::thread::JoinHandle<()> {
    let holder = std::fs::OpenOptions::new()
        .write(true)
        .share_mode(SHARE_WRITE_AND_DELETE)
        .open(path)
        .unwrap();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(held_ms));
        drop(holder);
    })
}

#[test]
fn a_state_file_held_a_moment_without_read_sharing_is_still_read() {
    let sandbox = Sandbox::new("held-state");
    let path = sandbox.path("state");
    std::fs::write(&path, "q5_pct@a 40\n").unwrap();
    let release = hold(&path, 5);
    let state = read_state(&path);
    release.join().unwrap();
    let pct = state.as_ref().and_then(|s| s.get("q5_pct@a"));
    assert_eq!(pct, Some("40"));
}

#[test]
fn a_state_file_unreadable_under_the_lock_keeps_the_other_keys() {
    let sandbox = Sandbox::new("held-save");
    let path = sandbox.path("state");
    std::fs::write(&path, "q5_pct@b 70\n").unwrap();
    let mut own = State::parse("");
    own.set("q5_pct@a", "40");
    let release = hold(&path, 400);
    assert!(save_state_at(&path, &own));
    release.join().unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "q5_pct@b 70\n");
    let parked = format!("{}.{}{}", path, std::process::id(), CHANGES_SUFFIX);
    assert_eq!(std::fs::read_to_string(parked).unwrap(), "q5_pct@a 40\n");
}
