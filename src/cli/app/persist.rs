use super::paths::state_path;
use crate::os::{self, Claim, Contents};
use crate::state::State;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

const STATE_LOCK_WAIT_MS: u32 = 500;
const STATE_LOCK_SLICE_MS: u32 = 25;
pub(super) const CHANGES_SUFFIX: &str = ".changes";
const STATE_READ_TRIES: u32 = 4;
const STATE_READ_RETRY_MS: u32 = 10;
pub(super) fn read_state(path: &str) -> Option<State> {
    for attempt in 0..STATE_READ_TRIES {
        if attempt > 0 {
            os::sleep(STATE_READ_RETRY_MS);
        }
        match os::read_checked(path, os::MAX_READ_BYTES) {
            Contents::Bytes(data) => return Some(State::parse(&String::from_utf8_lossy(&data))),
            Contents::Missing => return Some(State::parse("")),
            Contents::Unreadable => {}
        }
    }
    None
}

pub(super) fn pending_changes(path: &str) -> Vec<String> {
    let Some(dir) = os::parent(path) else {
        return Vec::new();
    };
    let prefix = format!("{}.", os::file_name(path));
    let mut found: Vec<(Option<i64>, String)> = os::list(dir, &prefix)
        .into_iter()
        .filter(|name| name.ends_with(CHANGES_SUFFIX))
        .map(|name| os::join(dir, &[&name]))
        .map(|file| (os::written(&file), file))
        .collect();
    found.sort();
    found.into_iter().map(|(_, file)| file).collect()
}

fn read_pending(path: &str) -> Vec<(String, String)> {
    pending_changes(path)
        .into_iter()
        .filter_map(|file| os::read_text(&file).map(|changes| (file, changes)))
        .collect()
}

fn with_pending(mut state: State, pending: &[(String, String)]) -> State {
    for (_, changes) in pending {
        state.apply(changes);
    }
    state
}

pub(super) fn load_state() -> State {
    let Some(path) = state_path() else {
        return State::parse("");
    };
    let pending = read_pending(&path);
    let state = read_state(&path).unwrap_or_else(|| State::parse(""));
    if pending.is_empty() {
        return state;
    }
    State::parse(&with_pending(state, &pending).text())
}

fn merge_state(path: &str, own: Option<&State>) -> bool {
    let pending = read_pending(path);
    let Some(latest) = read_state(path) else {
        return own.is_some_and(|state| park(path, state).is_some());
    };
    let mut merged = with_pending(latest, &pending);
    if let Some(state) = own {
        merged.apply(&state.changes());
    }
    let written = merged.changed() && os::write_atomic(path, merged.text().as_bytes());
    if written || !merged.changed() {
        for (file, _) in &pending {
            os::remove(file);
        }
    }
    written
}

fn park(path: &str, state: &State) -> Option<String> {
    let parked = format!("{}.{}{}", path, os::pid(), CHANGES_SUFFIX);
    let mut changes = os::read_text(&parked).unwrap_or_default();
    if !changes.is_empty() && !changes.ends_with('\n') {
        changes.push('\n');
    }
    changes.push_str(&state.changes());
    os::write_atomic(&parked, changes.as_bytes()).then_some(parked)
}

fn park_changes(path: &str, lock_path: &str, state: &State) -> bool {
    let Some(parked) = park(path, state) else {
        return false;
    };
    for _ in 0..STATE_LOCK_WAIT_MS / STATE_LOCK_SLICE_MS {
        if !os::exists(&parked) {
            break;
        }
        if let Claim::Held(_lock) = os::lock(lock_path, STATE_LOCK_SLICE_MS) {
            merge_state(path, None);
            break;
        }
    }
    true
}

pub(super) fn save_state(state: &State) -> bool {
    state_path().is_some_and(|path| save_state_at(&path, state))
}

pub(super) fn save_state_at(path: &str, state: &State) -> bool {
    if !state.changed() {
        return false;
    }
    if let Some(d) = os::parent(path) {
        if !os::exists(d) {
            os::mkdirs(d);
        }
    }
    let lock_path = format!("{}.lock", path);
    let claim = os::lock(&lock_path, 0);
    if let Claim::Busy = claim {
        return park_changes(path, &lock_path, state);
    }
    merge_state(path, Some(state))
}
