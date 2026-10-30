use super::{recent, State};
use crate::render::{Quota, Quotas};
use crate::session::Live;
use alloc::format;

const FRESH: f64 = 600.0;
const HOUR: f64 = 3600.0;
pub(super) const FIVE_HOUR_PERIOD: f64 = 5.0 * HOUR;
const SEVEN_DAY_PERIOD: f64 = 7.0 * 24.0 * HOUR;
const YEAR_2000: f64 = 946_684_800.0;
const YEAR_2100: f64 = 4_102_444_800.0;

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
}
