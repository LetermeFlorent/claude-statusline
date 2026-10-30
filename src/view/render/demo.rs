use super::SysValue;
use crate::config::{by_name, Config, Metric};
use alloc::vec::Vec;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Level {
    Low,
    Warn,
    Hot,
}

pub const LEVEL_NAMES: [(&str, Level); 3] = [
    ("low", Level::Low),
    ("warn", Level::Warn),
    ("hot", Level::Hot),
];

const GIB: f64 = 1_073_741_824.0;
const SAMPLE_MEMORY: f64 = 16.0 * GIB;

impl Level {
    pub fn named(text: &str) -> Option<Level> {
        by_name(&LEVEL_NAMES, text)
    }

    pub fn context_percent(self) -> u64 {
        match self {
            Level::Low => 34,
            Level::Warn => 74,
            Level::Hot => 93,
        }
    }

    fn disk(self) -> f64 {
        match self {
            Level::Low => 12.0,
            Level::Warn => 76.0,
            Level::Hot => 93.0,
        }
    }

    fn cpu(self) -> f64 {
        match self {
            Level::Low => 18.0,
            Level::Warn => 78.0,
            Level::Hot => 96.0,
        }
    }

    fn memory_left(self) -> f64 {
        match self {
            Level::Low => 0.65,
            Level::Warn => 0.22,
            Level::Hot => 0.08,
        }
    }
}

pub fn readings(cfg: &Config, level: Option<Level>, real: Option<(f64, f64)>) -> Vec<SysValue> {
    if !cfg.sys_enabled {
        return Vec::new();
    }
    let sample = level.unwrap_or(Level::Low);
    cfg.metrics
        .iter()
        .map(|metric| match metric {
            Metric::Ram => SysValue {
                metric: *metric,
                pct: None,
                ram: match level {
                    Some(level) => Some(shared(level, real)),
                    None => real,
                },
            },
            Metric::Io => SysValue {
                metric: *metric,
                pct: Some(sample.disk()),
                ram: None,
            },
            Metric::Cpu => SysValue {
                metric: *metric,
                pct: Some(sample.cpu()),
                ram: None,
            },
        })
        .collect()
}

fn shared(level: Level, real: Option<(f64, f64)>) -> (f64, f64) {
    let total = real.map_or(SAMPLE_MEMORY, |(total, _)| total);
    (total, total * level.memory_left())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pct(values: &[SysValue], metric: Metric) -> Option<f64> {
        values.iter().find(|v| v.metric == metric)?.pct
    }

    #[test]
    fn levels_are_named_in_any_case() {
        assert_eq!(Level::named("warn"), Some(Level::Warn));
        assert_eq!(Level::named(" HOT "), Some(Level::Hot));
        assert_eq!(Level::named("medium"), None);
    }

    #[test]
    fn the_disk_and_the_cpu_always_get_a_sample_value() {
        let cfg = Config::parse(None);
        let values = readings(&cfg, None, None);
        assert_eq!(pct(&values, Metric::Io), Some(12.0));
        assert_eq!(pct(&values, Metric::Cpu), Some(18.0));
    }

    #[test]
    fn the_levels_cross_the_default_thresholds() {
        let cfg = Config::parse(None);
        let cpu = |level| pct(&readings(&cfg, Some(level), None), Metric::Cpu).unwrap();
        assert!(cpu(Level::Low) <= cfg.sys_warn);
        assert!(cpu(Level::Warn) > cfg.sys_warn && cpu(Level::Warn) <= cfg.sys_hot);
        assert!(cpu(Level::Hot) > cfg.sys_hot);
    }

    #[test]
    fn memory_stays_real_without_a_level_and_follows_the_level_with_one() {
        let cfg = Config::parse(None);
        let real = Some((8.0 * GIB, 6.0 * GIB));
        let ram = |level| readings(&cfg, level, real)[0].ram;
        assert_eq!(ram(None), real);
        assert_eq!(ram(Some(Level::Hot)), Some((8.0 * GIB, 8.0 * GIB * 0.08)));
        let unknown = readings(&cfg, Some(Level::Low), None)[0].ram;
        assert_eq!(unknown.map(|(total, _)| total), Some(SAMPLE_MEMORY));
    }

    #[test]
    fn a_turned_off_second_line_stays_off() {
        let cfg = Config::parse(Some("{\"system\": {\"enabled\": false}}"));
        assert!(readings(&cfg, None, None).is_empty());
    }
}
