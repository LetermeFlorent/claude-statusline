use crate::config::Metric;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Quota {
    pub pct: Option<f64>,
    pub at: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quotas {
    pub five_hour: Option<Quota>,
    pub seven_day: Option<Quota>,
}

#[derive(Clone, Copy, Default)]
pub struct Ctx {
    pub size: Option<f64>,
    pub pct: Option<f64>,
    pub tokens: Option<u64>,
}

pub struct SysValue {
    pub metric: Metric,
    pub pct: Option<f64>,
    pub ram: Option<(f64, f64)>,
}

pub struct Data {
    pub ctx: Ctx,
    pub window: Option<u64>,
    pub quotas: Option<Quotas>,
    pub spend: Option<Quota>,
    pub cache_warm: Option<bool>,
    pub model: String,
    pub effort: Option<String>,
    pub sys: Vec<SysValue>,
    pub updating: bool,
}
