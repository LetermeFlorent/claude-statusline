use crate::render::Quota;
use crate::session::Live;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

// Fichier d'etat : une ligne "cle valeur" par entree, triees par cle.
// Les cles inconnues sont gardees telles quelles.
pub struct State {
    map: Vec<(String, String)>,
    loaded: String,
}

const FRESH: f64 = 600.0;
const HOUR: f64 = 3600.0;
const FIVE_HOUR_PERIOD: f64 = 5.0 * HOUR;
const SEVEN_DAY_PERIOD: f64 = 7.0 * 24.0 * HOUR;
const YEAR_2000: f64 = 946_684_800.0;
const YEAR_2100: f64 = 4_102_444_800.0;
const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

fn storable(key: &str, value: &str) -> bool {
    !key.is_empty()
        && !key.contains(|c: char| c.is_whitespace() || c == '\0')
        && !value.contains(['\n', '\r', '\0'])
}

pub fn fingerprint<'a>(parts: impl IntoIterator<Item = &'a str>) -> String {
    let mut hash = FNV_OFFSET;
    for part in parts {
        for byte in part.bytes().chain(core::iter::once(0)) {
            hash = (hash ^ byte as u64).wrapping_mul(FNV_PRIME);
        }
    }
    format!("{:016x}", hash)
}

impl State {
    pub fn parse(text: &str) -> State {
        let mut state = State {
            map: Vec::new(),
            loaded: String::new(),
        };
        for line in text.split('\n') {
            let line = line.strip_suffix('\r').unwrap_or(line);
            if let Some(i) = line.find(' ') {
                state.set(&line[..i], &line[i + 1..]);
            }
        }
        state.loaded = state.text();
        state
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.map
            .iter()
            .find(|(x, _)| x == key)
            .map(|(_, v)| v.as_str())
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
        self.text() != self.loaded
    }

    // Quotas : ceux de Claude Code s'il les envoie (et on les garde), sinon
    // ceux du fichier s'ils ont moins de 10 minutes, avances jusqu'au prochain reset.
    // Le fichier sert a tous les comptes : chaque cle porte le suffixe @<tag> du compte.
    pub fn quotas(
        &mut self,
        live: Option<(Option<Live>, Option<Live>)>,
        now: f64,
        tag: &str,
    ) -> Option<(Option<Quota>, Option<Quota>)> {
        if let Some((five, seven)) = live {
            for k in ["q_at", "q5_at", "q5_pct", "q7_at", "q7_pct"] {
                // Cles de la version 1, sans compte
                self.remove(k);
                self.remove(&format!("{}@{}", k, tag));
            }
            self.set(&format!("q_at@{}", tag), &format!("{}", now as i64));
            for (window, prefix) in [(five, "q5"), (seven, "q7")] {
                if let Some(at) = window.and_then(|w| w.at) {
                    self.set(&format!("{}_at@{}", prefix, tag), &format!("{}", at));
                    if let Some(pct) = window.and_then(|w| w.pct) {
                        self.set(&format!("{}_pct@{}", prefix, tag), &format!("{}", pct));
                    }
                }
            }
            return Some((five.map(|w| w.quota()), seven.map(|w| w.quota())));
        }
        let q_at = self.num(&format!("q_at@{}", tag))?;
        if !(now - q_at < FRESH) {
            return None;
        }
        let q5 = self.stored("q5", tag, FIVE_HOUR_PERIOD, now);
        let q7 = self.stored("q7", tag, SEVEN_DAY_PERIOD, now);
        if q5.is_none() && q7.is_none() {
            return None;
        }
        Some((q5, q7))
    }

    fn stored(&self, prefix: &str, tag: &str, period: f64, now: f64) -> Option<Quota> {
        let at = self
            .num(&format!("{}_at@{}", prefix, tag))
            .filter(|at| (YEAR_2000..YEAR_2100).contains(at))?;
        if at > now {
            let pct = self.num(&format!("{}_pct@{}", prefix, tag));
            return Some(Quota { pct, at: Some(at) });
        }
        let periods = crate::num::floor((now - at) / period) + 1.0;
        Some(Quota {
            pct: None,
            at: Some(at + periods * period),
        })
    }

    // Pourcentage d'un compteur systeme par difference avec le releve precedent
    pub fn counter(&mut self, name: &str, current: Option<(i64, i64)>) -> Option<f64> {
        let clock_key = format!("{}_clock", name);
        let value_key = format!("{}_value", name);
        let previous = (self.int(&clock_key), self.int(&value_key));
        let Some((clock, value)) = current else {
            self.remove(&clock_key);
            self.remove(&value_key);
            return None;
        };
        self.set(&clock_key, &format!("{}", clock));
        self.set(&value_key, &format!("{}", value));
        let (previous_clock, previous_value) = (previous.0?, previous.1?);
        let elapsed = clock.wrapping_sub(previous_clock);
        let busy = value.wrapping_sub(previous_value);
        if elapsed <= 0 || busy < 0 {
            return None;
        }
        let pct = busy as f64 / elapsed as f64 * 100.0;
        Some(if pct > 100.0 { 100.0 } else { pct })
    }
}

// Tag du compte tire de son dossier de config : ".claude-compte2" => "claude-compte2".
// Le hook SessionStart de claude-account-menu calcule le meme.
pub fn tag(dir: &str) -> String {
    let leaf = dir
        .trim_end_matches(['/', '\\'])
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("");
    let name: String = leaf
        .trim_start_matches('.')
        .chars()
        .map(|c| {
            let c = c.to_ascii_lowercase();
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if name.is_empty() {
        String::from("claude")
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fichier() {
        let mut s = State::parse("z 1\r\nq5_at  12\na b c\n");
        assert_eq!(s.get("a"), Some("b c"));
        assert_eq!(s.num("q5_at"), None);
        assert!(!s.changed());
        s.set("b", "2");
        assert_eq!(s.text(), "a b c\nb 2\nq5_at  12\nz 1\n");
        assert!(s.changed());
    }

    #[test]
    fn quotas() {
        let now = 1_800_000_000.0;
        let mut s = State::parse("q_at@a 1799999999\nq5_at@a 1799990000\nq5_pct@a 40\n");
        let (q5, q7) = s.quotas(None, now, "a").unwrap();
        assert_eq!(
            q5,
            Some(Quota {
                pct: None,
                at: Some(1_800_008_000.0)
            })
        );
        assert_eq!(q7, None);
        // Les quotas d'un autre compte ne servent pas
        assert!(s.quotas(None, now, "b").is_none());
        let mut s = State::parse("q_at@a 1\nq5_at@a 2000000\n");
        assert!(s.quotas(None, now, "a").is_none());
        let live = Some((
            Some(Live {
                pct: Some(10.5),
                at: Some(1791373904.7),
            }),
            None,
        ));
        let mut s = State::parse("q7_pct 3\nq7_pct@a 3\nq7_pct@b 4\nq7_x 2\nx 1\n");
        s.quotas(live, now, "a");
        assert_eq!(
            s.text(),
            "q5_at@a 1791373904.7\nq5_pct@a 10.5\nq7_pct@b 4\nq7_x 2\nq_at@a 1800000000\nx 1\n"
        );
    }

    #[test]
    fn absurd_reset_times_are_dropped_without_hanging() {
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut state =
                State::parse("q_at@claude 1800000000\nq5_at@claude -5e38\nq7_at@claude 1e300\n");
            let _ = sender.send(state.quotas(None, 1_800_000_100.0, "claude"));
        });
        let quotas = receiver.recv_timeout(std::time::Duration::from_secs(2));
        assert_eq!(quotas, Ok(None));
    }

    #[test]
    fn values_cannot_add_lines() {
        let mut state = State::parse("");
        state.set("bg_stamp", "x\nq5_at@claude -5e38");
        state.set("bg_host", "dark\r");
        state.set("bad key", "1");
        state.set("nul\0", "1");
        state.set("ok", "a b");
        assert_eq!(State::parse(&state.text()).text(), "ok a b\n");
    }

    #[test]
    fn non_finite_numbers_read_as_missing() {
        let state = State::parse("a inf\nb NaN\nc 1e999\nd 2.5\n");
        let read = ["a", "b", "c", "d"].map(|k| state.num(k));
        assert_eq!(read, [None, None, None, Some(2.5)]);
    }

    #[test]
    fn comptes() {
        assert_eq!(tag("C:\\Users\\u\\.claude"), "claude");
        assert_eq!(tag("/home/u/.claude-compte2/"), "claude-compte2");
        assert_eq!(tag("/home/u/.Claude Work/"), "claude_work");
        assert_eq!(tag(""), "claude");
    }

    #[test]
    fn compteurs() {
        let mut s = State::parse("cpu_clock 100\ncpu_value 10\n");
        assert_eq!(s.counter("cpu", Some((200, 60))), Some(50.0));
        assert_eq!(s.counter("cpu", Some((200, 70))), None);
        assert_eq!(s.counter("io", None), None);
        assert_eq!(s.get("cpu_value"), Some("70"));
    }
}
