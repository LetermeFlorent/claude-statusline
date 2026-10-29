use crate::os::*;
use crate::sandbox::Sandbox;

#[test]
fn a_held_lock_turns_others_away_within_the_wait() {
    const WAIT_MS: u32 = 50;
    const SURELY_WAITED_MS: u64 = 25;
    const BOUND_MS: u64 = 1000;
    const RELEASE_WAIT_MS: u32 = 1000;
    let sandbox = Sandbox::new("lock");
    let path = sandbox.path("state.lock");
    let held = lock(&path, 0);
    assert!(matches!(held, Claim::Held(_)));
    let start = std::time::Instant::now();
    assert!(matches!(lock(&path, WAIT_MS), Claim::Busy));
    let waited = start.elapsed().as_millis() as u64;
    assert!((SURELY_WAITED_MS..BOUND_MS).contains(&waited), "{}", waited);
    drop(held);
    assert!(matches!(lock(&path, RELEASE_WAIT_MS), Claim::Held(_)));
}
