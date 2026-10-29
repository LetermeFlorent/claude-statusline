use super::Metric;
use crate::json::Value;
use alloc::vec::Vec;

pub const BACKGROUND_NAMES: [(&str, bool); 4] = [
    ("light", true),
    ("clair", true),
    ("dark", false),
    ("sombre", false),
];
pub const EFFORT_NAMES: [(&str, bool); 2] = [("gradient", false), ("flat", true)];
pub const METRIC_NAMES: [(&str, Metric); 4] = [
    ("ram", Metric::Ram),
    ("io", Metric::Io),
    ("dsk", Metric::Io),
    ("cpu", Metric::Cpu),
];
const DEFAULT_METRICS: [Metric; 3] = [Metric::Ram, Metric::Io, Metric::Cpu];

pub fn by_name<T: Copy>(names: &[(&str, T)], text: &str) -> Option<T> {
    let word = text.trim().to_lowercase();
    names
        .iter()
        .find(|(name, _)| *name == word)
        .map(|(_, value)| *value)
}

pub fn background_word(s: &str) -> Option<bool> {
    by_name(&BACKGROUND_NAMES, s)
}

pub(super) fn listed_metrics(list: Option<&Value>) -> Vec<Metric> {
    let named = list.and_then(Value::arr).unwrap_or_default();
    let mut picked = Vec::new();
    for metric in named
        .iter()
        .filter_map(|m| by_name(&METRIC_NAMES, m.str()?))
    {
        if !picked.contains(&metric) {
            picked.push(metric);
        }
    }
    if picked.is_empty() {
        DEFAULT_METRICS.to_vec()
    } else {
        picked
    }
}
