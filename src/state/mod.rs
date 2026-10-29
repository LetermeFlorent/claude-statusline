use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

mod account;
mod counter;
mod hash;
mod quotas;
#[cfg(test)]
mod tests;

pub use account::account_tag;
pub use hash::fingerprint;

pub struct State {
    map: Vec<(String, String)>,
    loaded: Vec<(String, String)>,
}

const BYTE_ORDER_MARK: char = '\u{feff}';

fn storable(key: &str, value: &str) -> bool {
    !key.is_empty()
        && !key.contains(|c: char| c.is_whitespace() || c == '\0')
        && !value.contains(['\n', '\r', '\0'])
}

pub fn recent(at: f64, now: f64, span: f64) -> bool {
    now >= at && now - at < span
}

fn value_in<'a>(map: &'a [(String, String)], key: &str) -> Option<&'a str> {
    map.iter().find(|(x, _)| x == key).map(|(_, v)| v.as_str())
}

impl State {
    pub fn parse(text: &str) -> State {
        let mut state = State {
            map: Vec::new(),
            loaded: Vec::new(),
        };
        let text = text.strip_prefix(BYTE_ORDER_MARK).unwrap_or(text);
        for line in text.split('\n') {
            let line = line.strip_suffix('\r').unwrap_or(line);
            if let Some(i) = line.find(' ') {
                state.set(&line[..i], &line[i + 1..]);
            }
        }
        state.loaded = state.map.clone();
        state
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        value_in(&self.map, key)
    }

    pub fn num(&self, key: &str) -> Option<f64> {
        self.get(key)?.parse().ok().filter(|n: &f64| n.is_finite())
    }

    pub fn int(&self, key: &str) -> Option<i64> {
        self.get(key)?.parse().ok()
    }

    pub fn set(&mut self, key: &str, value: &str) {
        if !storable(key, value) {
            return;
        }
        match self.map.binary_search_by(|(x, _)| x.as_str().cmp(key)) {
            Ok(i) => self.map[i].1 = String::from(value),
            Err(i) => self.map.insert(i, (String::from(key), String::from(value))),
        }
    }

    pub fn remove(&mut self, key: &str) {
        self.map.retain(|(x, _)| x != key);
    }

    pub fn with_prefix<'a>(
        &'a self,
        prefix: &'a str,
    ) -> impl Iterator<Item = (&'a str, &'a str)> + 'a {
        let keys = self.map.iter().filter(move |(x, _)| x.starts_with(prefix));
        keys.map(|(x, v)| (x.as_str(), v.as_str()))
    }

    pub fn text(&self) -> String {
        let mut out = String::new();
        for (key, value) in &self.map {
            out.push_str(key);
            out.push(' ');
            out.push_str(value);
            out.push('\n');
        }
        out
    }

    pub fn changed(&self) -> bool {
        self.map != self.loaded
    }

    pub fn changes(&self) -> String {
        let mut out = String::new();
        for (key, value) in &self.map {
            if value_in(&self.loaded, key) != Some(value.as_str()) {
                out.push_str(&format!("{} {}\n", key, value));
            }
        }
        for (key, _) in &self.loaded {
            if self.get(key).is_none() {
                out.push_str(&format!("{}\n", key));
            }
        }
        out
    }

    pub fn apply(&mut self, changes: &str) {
        for line in changes.lines() {
            match line.split_once(' ') {
                Some((key, value)) => self.set(key, value),
                None => self.remove(line),
            }
        }
    }
}
