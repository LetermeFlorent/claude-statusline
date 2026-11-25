use super::quotas::{FIVE_HOUR_PERIOD, SEVEN_DAY_PERIOD};
use alloc::format;
use alloc::string::String;

pub(super) struct WindowKeys {
    pub(super) at: &'static str,
    pub(super) pct: &'static str,
    pub(super) period: f64,
}

pub(super) const FIVE_HOUR: WindowKeys = WindowKeys {
    at: "q5_at",
    pct: "q5_pct",
    period: FIVE_HOUR_PERIOD,
};
pub(super) const SEVEN_DAY: WindowKeys = WindowKeys {
    at: "q7_at",
    pct: "q7_pct",
    period: SEVEN_DAY_PERIOD,
};
pub(super) const STAMP: &str = "q_at";
pub(super) const NO_FIVE: &str = "q5_none";
pub(super) const WRITTEN: [&str; 5] = [
    FIVE_HOUR.at,
    FIVE_HOUR.pct,
    NO_FIVE,
    SEVEN_DAY.at,
    SEVEN_DAY.pct,
];

pub(super) fn tagged(key: &str, tag: &str) -> String {
    format!("{}@{}", key, tag)
}
