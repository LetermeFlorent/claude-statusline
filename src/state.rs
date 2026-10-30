use crate::os::Activity;
use crate::render::{Quota, Quotas};
use crate::session::Live;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub struct State {
    map: Vec<(String, String)>,
    loaded: Vec<(String, String)>,
}

const FRESH: f64 = 600.0;
const SAME_READING: f64 = 1.0;
const STALE_READING: f64 = 60.0;
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

pub fn recent(at: f64, now: f64, span: f64) -> bool {
    now >= at && now - at < span
}

fn quota_as_of(quota: Quota, period: f64, now: f64) -> Quota {
    let Some(at) = quota.at.filter(|at| (YEAR_2000..YEAR_2100).contains(at)) else {
        return Quota { at: None, ..quota };
    };
    if at > now {
        return quota;
    }
    let periods = crate::num::floor((now - at) / period) + 1.0;
    Quota {
        pct: None,
        at: Some(at + periods * period),
    }
}

fn value_in<'a>(map: &'a [(String, String)], key: &str) -> Option<&'a str> {
    map.iter().find(|(x, _)| x == key).map(|(_, v)| v.as_str())
}

fn percent(previous: Activity, reading: Activity) -> Option<f64> {
    let elapsed = reading.clock.wrapping_sub(previous.clock);
    let busy = reading.busy.wrapping_sub(previous.busy);
    if elapsed <= 0 || busy < 0 {
        return None;
    }
    let pct = busy as f64 / elapsed as f64 * 100.0;
    Some(if pct > 100.0 { 100.0 } else { pct })
}

impl State {
    pub fn parse(text: &str) -> State {
        let mut state = State {
            map: Vec::new(),
            loaded: Vec::new(),
        };
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

    pub fn rebased_on(&self, mut latest: State) -> State {
        for (key, value) in &self.map {
            if value_in(&self.loaded, key) != Some(value.as_str()) {
                latest.set(key, value);
            }
        }
        for (key, _) in &self.loaded {
            if self.get(key).is_none() {
                latest.remove(key);
            }
        }
        latest
    }

    pub fn quotas(
        &mut self,
        live: Option<(Option<Live>, Option<Live>)>,
        now: f64,
        tag: &str,
    ) -> Option<Quotas> {
        if let Some((five, seven)) = live {
            for key in ["q_at", "q5_at", "q5_pct", "q7_at", "q7_pct"] {
                self.remove(key);
                self.remove(&format!("{}@{}", key, tag));
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
            return Some(Quotas {
                five_hour: five.map(|w| quota_as_of(w.quota(), FIVE_HOUR_PERIOD, now)),
                seven_day: seven.map(|w| quota_as_of(w.quota(), SEVEN_DAY_PERIOD, now)),
            });
        }
        self.num(&format!("q_at@{}", tag))
            .filter(|at| recent(*at, now, FRESH))?;
        let five_hour = self.stored("q5", tag, FIVE_HOUR_PERIOD, now);
        let seven_day = self.stored("q7", tag, SEVEN_DAY_PERIOD, now);
        if five_hour.is_none() && seven_day.is_none() {
            return None;
        }
        Some(Quotas {
            five_hour,
            seven_day,
        })
    }

    fn stored(&self, prefix: &str, tag: &str, period: f64, now: f64) -> Option<Quota> {
        let at = self.num(&format!("{}_at@{}", prefix, tag));
        let pct = self.num(&format!("{}_pct@{}", prefix, tag));
        Some(quota_as_of(Quota { pct, at }, period, now)).filter(|quota| quota.at.is_some())
    }

    pub fn counter(&mut self, name: &str, current: Option<Activity>, now: f64) -> Option<f64> {
        let keys =
            ["source", "clock", "value", "at", "pct"].map(|part| format!("{}_{}", name, part));
        let [source_key, clock_key, value_key, at_key, pct_key] = &keys;
        let Some(reading) = current else {
            keys.iter().for_each(|key| self.remove(key));
            return None;
        };
        let source = format!("{}", reading.source);
        let at = self
            .num(at_key)
            .filter(|_| self.get(source_key) == Some(source.as_str()));
        if at.is_some_and(|at| recent(at, now, SAME_READING)) {
            return self.num(pct_key);
        }
        let previous = self
            .int(clock_key)
            .zip(self.int(value_key))
            .filter(|_| at.is_some_and(|at| recent(at, now, STALE_READING)))
            .map(|(clock, busy)| Activity {
                clock,
                busy,
                ..reading
            });
        self.set(source_key, &source);
        self.set(clock_key, &format!("{}", reading.clock));
        self.set(value_key, &format!("{}", reading.busy));
        self.set(at_key, &format!("{}", now));
        let pct = previous.and_then(|previous| percent(previous, reading));
        match pct {
            Some(pct) => self.set(pct_key, &format!("{}", pct)),
            None => self.remove(pct_key),
        }
        pct
    }
}

pub fn account_tag(dir: &str) -> String {
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
    fn lines_come_back_sorted_and_changes_are_tracked() {
        let mut state = State::parse("z 1\r\nq5_at  12\na b c\n");
        assert_eq!(state.get("a"), Some("b c"));
        assert_eq!(state.num("q5_at"), None);
        assert!(!state.changed());
        state.set("b", "2");
        assert_eq!(state.text(), "a b c\nb 2\nq5_at  12\nz 1\n");
        assert!(state.changed());
    }

    #[test]
    fn saving_keeps_what_another_process_wrote_since_loading() {
        let mut mine = State::parse("bg_stamp 1\nq5_pct@a 10\nq_at@a 5\n");
        mine.set("q5_pct@a", "20");
        mine.remove("q_at@a");
        mine.set("cpu_at", "7");
        let latest = State::parse("bg_stamp 2\nq5_pct@a 10\nq5_pct@b 30\nq_at@a 5\n");
        let saved = mine.rebased_on(latest);
        assert_eq!(
            saved.text(),
            "bg_stamp 2\ncpu_at 7\nq5_pct@a 20\nq5_pct@b 30\n"
        );
        assert!(saved.changed());
    }

    #[test]
    fn a_state_set_back_to_what_it_loaded_changes_nothing_on_disk() {
        let mut mine = State::parse("q_at@a 5\n");
        mine.remove("q_at@a");
        mine.set("q_at@a", "5");
        let saved = mine.rebased_on(State::parse("q_at@a 6\n"));
        assert_eq!(saved.text(), "q_at@a 6\n");
        assert!(!saved.changed());
    }

    #[test]
    fn quotas() {
        let now = 1_800_000_000.0;
        let mut state = State::parse("q_at@a 1799999999\nq5_at@a 1799990000\nq5_pct@a 40\n");
        let quotas = state.quotas(None, now, "a").unwrap();
        assert_eq!(
            quotas.five_hour,
            Some(Quota {
                pct: None,
                at: Some(1_800_008_000.0)
            })
        );
        assert_eq!(quotas.seven_day, None);
        assert!(state.quotas(None, now, "b").is_none());
        let mut state = State::parse("q_at@a 1\nq5_at@a 2000000\n");
        assert!(state.quotas(None, now, "a").is_none());
        let live = Some((
            Some(Live {
                pct: Some(10.5),
                at: Some(1791373904.7),
            }),
            None,
        ));
        let mut state = State::parse("q7_pct 3\nq7_pct@a 3\nq7_pct@b 4\nq7_x 2\nx 1\n");
        state.quotas(live, now, "a");
        assert_eq!(
            state.text(),
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
    fn account_tag_comes_from_the_config_folder() {
        assert_eq!(account_tag("C:\\Users\\u\\.claude"), "claude");
        assert_eq!(account_tag("/home/u/.claude-compte2/"), "claude-compte2");
        assert_eq!(account_tag("/home/u/.Claude Work/"), "claude_work");
        assert_eq!(account_tag(""), "claude");
    }

    fn reading(source: u64, clock: i64, busy: i64) -> Option<Activity> {
        Some(Activity {
            source,
            clock,
            busy,
        })
    }

    #[test]
    fn counter_percent_comes_from_the_previous_reading() {
        let mut state = State::parse("cpu_source 0\ncpu_clock 100\ncpu_value 10\ncpu_at 1000\n");
        assert_eq!(
            state.counter("cpu", reading(0, 200, 60), 1003.0),
            Some(50.0)
        );
        assert_eq!(state.counter("cpu", reading(0, 200, 70), 1006.0), None);
        assert_eq!(state.counter("io", None, 1006.0), None);
        assert_eq!(state.get("cpu_value"), Some("70"));
    }

    #[test]
    fn a_reading_from_another_source_starts_over() {
        let mut state = State::parse("");
        assert_eq!(state.counter("io", reading(1, 100, 10), 1000.0), None);
        assert_eq!(state.counter("io", reading(2, 200, 90), 1003.0), None);
        assert_eq!(
            state.counter("io", reading(2, 300, 140), 1006.0),
            Some(50.0)
        );
    }

    #[test]
    fn close_readings_keep_the_reference_and_late_ones_give_no_percent() {
        let mut state = State::parse("");
        assert_eq!(state.counter("cpu", reading(0, 100, 10), 1000.0), None);
        assert_eq!(
            state.counter("cpu", reading(0, 200, 60), 1003.0),
            Some(50.0)
        );
        assert_eq!(
            state.counter("cpu", reading(0, 201, 61), 1003.01),
            Some(50.0)
        );
        assert_eq!(state.get("cpu_clock"), Some("200"));
        assert_eq!(state.counter("cpu", reading(0, 300, 160), 1123.0), None);
        assert_eq!(
            state.counter("cpu", reading(0, 400, 210), 1126.0),
            Some(50.0)
        );
    }

    #[test]
    fn a_live_quota_past_its_reset_loses_its_percentage() {
        let now = 1_800_000_000.0;
        let live = Live {
            pct: Some(85.0),
            at: Some(now - 600.0),
        };
        let mut state = State::parse("");
        let quotas = state.quotas(Some((Some(live), None)), now, "a").unwrap();
        let at = Some(now - 600.0 + FIVE_HOUR_PERIOD);
        assert_eq!(quotas.five_hour, Some(Quota { pct: None, at }));
        assert_eq!(state.get("q5_pct@a"), Some("85"));
    }

    #[test]
    fn times_stamped_in_the_future_are_not_recent() {
        assert!(recent(100.0, 129.0, 30.0));
        assert!(!recent(100.0, 130.0, 30.0));
        assert!(!recent(100.0, 99.0, 30.0));
        let mut state = State::parse("q_at@a 1800000600\nq5_at@a 1800010000\nq5_pct@a 40\n");
        assert!(state.quotas(None, 1_800_000_000.0, "a").is_none());
    }
}
