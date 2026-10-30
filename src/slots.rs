use crate::state::State;
use alloc::string::String;
use alloc::vec::Vec;

pub fn prune(state: &mut State, prefix: &str, slots: usize, at_of: &dyn Fn(&str) -> Option<i64>) {
    let mut kept: Vec<(i64, String)> = Vec::new();
    let mut broken: Vec<String> = Vec::new();
    for (key, value) in state.with_prefix(prefix) {
        match at_of(value) {
            Some(at) => kept.push((at, String::from(key))),
            None => broken.push(String::from(key)),
        }
    }
    kept.sort();
    let surplus = kept.len().saturating_sub(slots);
    let evicted = kept.into_iter().take(surplus).map(|(_, key)| key);
    for key in broken.into_iter().chain(evicted) {
        state.remove(&key);
    }
}
