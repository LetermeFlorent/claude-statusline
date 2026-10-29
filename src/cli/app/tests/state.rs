use crate::app::persist::read_state;
use crate::sandbox::Sandbox;

#[test]
fn a_state_file_held_a_moment_without_read_sharing_is_still_read() {
    use std::os::windows::fs::OpenOptionsExt;
    const SHARE_WRITE_AND_DELETE: u32 = 6;
    const HELD_MS: u64 = 5;
    let sandbox = Sandbox::new("held-state");
    let path = sandbox.path("state");
    std::fs::write(&path, "q5_pct@a 40\n").unwrap();
    let holder = std::fs::OpenOptions::new()
        .write(true)
        .share_mode(SHARE_WRITE_AND_DELETE)
        .open(&path)
        .unwrap();
    let release = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(HELD_MS));
        drop(holder);
    });
    let state = read_state(&path);
    release.join().unwrap();
    let pct = state.as_ref().and_then(|s| s.get("q5_pct@a"));
    assert_eq!(pct, Some("40"));
}
