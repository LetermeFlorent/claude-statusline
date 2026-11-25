use super::{recent, State};
use crate::render::{Quota, Quotas};
use crate::session::Live;
use alloc::format;
use alloc::string::String;

const FRESH: f64 = 600.0;
const STAMP_LAG: f64 = 10.0;
const SAME_WINDOW: f64 = 60.0;
const HOUR: f64 = 3600.0;
pub(super) const FIVE_HOUR_PERIOD: f64 = 5.0 * HOUR;
const SEVEN_DAY_PERIOD: f64 = 7.0 * 24.0 * HOUR;
const YEAR_2000: f64 = 946_684_800.0;
const YEAR_2100: f64 = 4_102_444_800.0;

struct WindowKeys {
    at: &'static str,
    pct: &'static str,
}

const FIVE_HOUR: WindowKeys = WindowKeys {
    at: "q5_at",
    pct: "q5_pct",
};
const SEVEN_DAY: WindowKeys = WindowKeys {
    at: "q7_at",
    pct: "q7_pct",
};
const STAMP: &str = "q_at";
const NO_FIVE: &str = "q5_none";
const WRITTEN: [&str; 5] = [
    FIVE_HOUR.at,
    FIVE_HOUR.pct,
    NO_FIVE,
    SEVEN_DAY.at,
    SEVEN_DAY.pct,
];

fn tagged(key: &str, tag: &str) -> String {
    format!("{}@{}", key, tag)
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

impl State {
    pub fn quotas(
        &mut self,
        live: Option<(Option<Live>, Option<Live>)>,
        now: f64,
        tag: &str,
    ) -> Option<Quotas> {
        if let Some((five, seven)) = live {
            let stamp_key = tagged(STAMP, tag);
            let stamp = self.get(&stamp_key).map(String::from);
            let before = self.written(tag);
            let five = self.merged(five, &FIVE_HOUR, tag);
            let seven = self.merged(seven, &SEVEN_DAY, tag);
            for key in core::iter::once(STAMP).chain(WRITTEN) {
                self.remove(key);
                self.remove(&tagged(key, tag));
            }
            if five.is_none() && seven.is_some() {
                self.set(&tagged(NO_FIVE, tag), "1");
            }
            for (window, keys) in [(five, &FIVE_HOUR), (seven, &SEVEN_DAY)] {
                if let Some(at) = window.and_then(|w| w.at) {
                    self.set(&tagged(keys.at, tag), &format!("{}", at));
                    if let Some(pct) = window.and_then(|w| w.pct) {
                        self.set(&tagged(keys.pct, tag), &format!("{}", pct));
                    }
                }
            }
            let kept_stamp = stamp.filter(|at| {
                before == self.written(tag)
                    && at.parse::<f64>().is_ok_and(|at| recent(at, now, STAMP_LAG))
            });
            self.set(
                &stamp_key,
                &kept_stamp.unwrap_or_else(|| format!("{}", now as i64)),
            );
            return Some(Quotas {
                five_hour: five.map(|w| quota_as_of(w.quota(), FIVE_HOUR_PERIOD, now)),
                seven_day: seven.map(|w| quota_as_of(w.quota(), SEVEN_DAY_PERIOD, now)),
            });
        }
        self.num(&tagged(STAMP, tag))
            .filter(|at| recent(*at, now, FRESH))?;
        let five_hour = self.stored(&FIVE_HOUR, tag, FIVE_HOUR_PERIOD, now);
        let seven_day = self.stored(&SEVEN_DAY, tag, SEVEN_DAY_PERIOD, now);
        if five_hour.is_none() && seven_day.is_none() {
            return None;
        }
        let no_five = self.num(&tagged(NO_FIVE, tag)).is_some();
        Some(Quotas {
            five_hour: if no_five {
                None
            } else {
                Some(five_hour.unwrap_or_default())
            },
            seven_day,
        })
    }

    fn merged(&self, window: Option<Live>, keys: &WindowKeys, tag: &str) -> Option<Live> {
        let window = window?;
        let (Some(at), Some(stored_at)) = (window.at, self.num(&tagged(keys.at, tag))) else {
            return Some(window);
        };
        let pct = self.num(&tagged(keys.pct, tag));
        if stored_at > at + SAME_WINDOW {
            return Some(Live {
                pct,
                at: Some(stored_at),
            });
        }
        if stored_at < at - SAME_WINDOW {
            return Some(window);
        }
        Some(Live {
            pct: match (window.pct, pct) {
                (Some(new), Some(old)) if new > 0.0 => Some(new.max(old)),
                (Some(new), _) => Some(new),
                (None, old) => old,
            },
            at: Some(stored_at),
        })
    }

    fn written(&self, tag: &str) -> [Option<String>; 5] {
        WRITTEN.map(|key| self.get(&tagged(key, tag)).map(String::from))
    }

    fn stored(&self, keys: &WindowKeys, tag: &str, period: f64, now: f64) -> Option<Quota> {
        let at = self.num(&tagged(keys.at, tag));
        let pct = self.num(&tagged(keys.pct, tag));
        Some(quota_as_of(Quota { pct, at }, period, now)).filter(|quota| quota.at.is_some())
    }
}
