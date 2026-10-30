use super::{Decision, Source};
use crate::slots;
use crate::state::{recent, State};
use alloc::format;
use alloc::string::String;

const PREFIX: &str = "bg@";
const SLOTS: usize = 4;
const TOUCH_AFTER: f64 = 3600.0;
const LIGHT: &str = "light";
const DARK: &str = "dark";
const LEGACY_KEYS: [&str; 4] = ["bg_host", "bg_source", "bg_stamp", "bg_at"];
const FIELDS: usize = 3;

struct Entry {
    at: i64,
    decision: Decision,
}

impl Entry {
    fn encode(&self) -> String {
        let word = if self.decision.light { LIGHT } else { DARK };
        format!("{} {} {}", self.at, word, self.decision.source.name())
    }

    fn decode(text: &str) -> Option<Entry> {
        let mut fields = text.splitn(FIELDS, ' ');
        let at = fields.next()?.parse().ok()?;
        let light = match fields.next()? {
            LIGHT => true,
            DARK => false,
            _ => return None,
        };
        let name = fields.next().filter(|name| !name.is_empty())?;
        let source = Source::from_name(name);
        Some(Entry {
            at,
            decision: Decision { light, source },
        })
    }
}

fn key(stamp: &str) -> String {
    format!("{}{}", PREFIX, stamp)
}

fn store(state: &mut State, key: &str, entry: &Entry) {
    state.set(key, &entry.encode());
    slots::prune(state, PREFIX, SLOTS, &|value| {
        Entry::decode(value).map(|entry| entry.at)
    });
}

pub fn recall(state: &mut State, stamp: &str, now: f64) -> Option<Decision> {
    let key = key(stamp);
    let entry = state.get(&key).and_then(Entry::decode)?;
    if !recent(entry.at as f64, now, TOUCH_AFTER) {
        let decision = entry.decision;
        store(
            state,
            &key,
            &Entry {
                at: now as i64,
                decision,
            },
        );
    }
    Some(entry.decision)
}

pub fn remember(state: &mut State, stamp: &str, decision: Decision, now: f64) {
    let entry = Entry {
        at: now as i64,
        decision,
    };
    store(state, &key(stamp), &entry);
}

pub fn drop_legacy(state: &mut State) {
    for key in LEGACY_KEYS {
        state.remove(key);
    }
}

#[cfg(test)]
mod tests;
