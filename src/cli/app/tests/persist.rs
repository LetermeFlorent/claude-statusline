use crate::app::persist::{pending_changes, read_state, save_state_at, CHANGES_SUFFIX};
use crate::os::{self, Claim};
use crate::sandbox::Sandbox;
use crate::state::State;
use std::time::{Duration, SystemTime};

fn ago(seconds: u64) -> SystemTime {
    SystemTime::now() - Duration::from_secs(seconds)
}

fn parked(path: &str, pid: u32, changes: &str, at: SystemTime) -> String {
    let file = format!("{}.{}{}", path, pid, CHANGES_SUFFIX);
    std::fs::write(&file, changes).unwrap();
    let opened = std::fs::File::options().write(true).open(&file).unwrap();
    opened.set_modified(at).unwrap();
    file
}

fn names(files: Vec<String>) -> Vec<String> {
    files.iter().map(|f| os::file_name(f).to_string()).collect()
}

fn own_change(key: &str, value: &str) -> State {
    let mut state = State::parse("");
    state.set(key, value);
    state
}

#[test]
fn parked_changes_come_in_the_order_they_were_written() {
    let sandbox = Sandbox::new("parked-order");
    let path = sandbox.path("state");
    parked(&path, 9, "q5_pct@x 30\n", ago(20));
    parked(&path, 10, "q5_pct@x 50\n", ago(10));
    let order = names(pending_changes(&path));
    assert_eq!(order, ["state.9.changes", "state.10.changes"]);
}

#[test]
fn parked_changes_written_together_come_in_name_order() {
    let sandbox = Sandbox::new("parked-tie");
    let path = sandbox.path("state");
    let at = ago(10);
    parked(&path, 9, "q5_pct@x 30\n", at);
    parked(&path, 10, "q5_pct@x 50\n", at);
    let order = names(pending_changes(&path));
    assert_eq!(order, ["state.10.changes", "state.9.changes"]);
}

#[test]
fn the_newest_parked_value_wins_the_merge() {
    let sandbox = Sandbox::new("parked-merge");
    let path = sandbox.path("state");
    parked(&path, 9, "q5_pct@x 30\n", ago(20));
    parked(&path, 10, "q5_pct@x 50\n", ago(10));
    assert!(save_state_at(&path, &own_change("q_at@y", "1")));
    let saved = read_state(&path).unwrap();
    assert_eq!(saved.get("q5_pct@x"), Some("50"));
    assert_eq!(sandbox.files(), ["state", "state.lock"]);
}

#[test]
fn parking_again_keeps_the_changes_not_merged_yet() {
    let sandbox = Sandbox::new("parked-reuse");
    let path = sandbox.path("state");
    let own = parked(&path, std::process::id(), "q7_pct@x 10\nq5_pct@x 20\n", ago(0));
    let Claim::Held(_lock) = os::lock(&format!("{}.lock", path), 0) else {
        panic!("lock not taken");
    };
    assert!(save_state_at(&path, &own_change("q5_pct@x", "30")));
    let mut merged = State::parse("");
    merged.apply(&std::fs::read_to_string(&own).unwrap());
    assert_eq!(merged.get("q7_pct@x"), Some("10"));
    assert_eq!(merged.get("q5_pct@x"), Some("30"));
}
