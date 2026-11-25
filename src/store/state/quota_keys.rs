use alloc::format;
use alloc::string::String;

const HOUR: f64 = 3600.0;
const FIVE_HOUR_PERIOD: f64 = 5.0 * HOUR;
const SEVEN_DAY_PERIOD: f64 = 7.0 * 24.0 * HOUR;

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
