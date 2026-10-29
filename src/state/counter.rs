use super::{recent, State};
use crate::os::Activity;
use alloc::format;

const SAME_READING: f64 = 1.0;
const STALE_READING: f64 = 60.0;

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
