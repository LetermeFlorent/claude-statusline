use super::settings::Setting;
use crate::slots;
use crate::state::State;
use alloc::format;
use alloc::string::String;

const PREFIX: &str = "compact@";
const SLOTS: usize = 4;
const NO_SETTING: &str = "-";
const LEGACY_KEYS: [&str; 4] = [
    "compact_key",
    "compact_stamp",
    "compact_at",
    "compact_window",
];

pub struct Entry {
    pub at: i64,
    pub stamp: String,
    pub setting: Option<Setting>,
}

fn is_stamp(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, ':' | ';' | '-'))
}

impl Entry {
    fn encode(&self) -> String {
        let setting = self
            .setting
            .map_or(String::from(NO_SETTING), Setting::store);
        format!("{} {} {}", self.at, self.stamp, setting)
    }

    fn decode(text: &str) -> Option<Entry> {
        let mut fields = text.split(' ');
        let at = fields.next()?.parse().ok()?;
        let stamp = fields.next().filter(|stamp| is_stamp(stamp))?;
        let window = fields.next()?;
        if fields.next().is_some() {
            return None;
        }
        let setting = match window {
            NO_SETTING => None,
            _ => Some(Setting::load(window)?),
        };
        Some(Entry {
            at,
            stamp: String::from(stamp),
            setting,
        })
    }
}

pub fn key(files_key: &str) -> String {
    format!("{}{}", PREFIX, files_key)
}

pub fn load(state: &State, key: &str) -> Option<Entry> {
    state.get(key).and_then(Entry::decode)
}

pub fn save(state: &mut State, key: &str, entry: &Entry) {
    state.set(key, &entry.encode());
    prune(state);
}

pub fn drop_legacy(state: &mut State) {
    for key in LEGACY_KEYS {
        state.remove(key);
    }
}

fn prune(state: &mut State) {
    slots::prune(state, PREFIX, SLOTS, &|value| {
        Entry::decode(value).map(|entry| entry.at)
    });
}
