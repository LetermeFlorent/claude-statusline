use super::names::{background_word, by_name, EFFORT_NAMES};
use super::Config;
use crate::json::Value;
use crate::num;
use crate::render::printable;
use alloc::string::String;

type Path = &'static [&'static str];

fn unit(v: f64) -> f64 {
    num::clamp(v, 0.0, 1.0)
}

fn bar_width(config: &mut Config, v: f64) {
    config.width = (num::trunc(v) as i64).clamp(1, 40) as usize;
}

fn disk_path(config: &mut Config, v: &str) {
    if !v.is_empty() {
        config.disk_path = String::from(v);
    }
}

const FLAGS: [(Path, fn(&mut Config, bool)); 8] = [
    (&["bold"], |c, v| c.bold = v),
    (&["cold_cache_marker"], |c, v| c.cold_cache_marker = v),
    (&["segments", "ctx"], |c, v| c.seg_ctx = v),
    (&["segments", "5h"], |c, v| c.seg_5h = v),
    (&["segments", "7d"], |c, v| c.seg_7d = v),
    (&["segments", "spend"], |c, v| c.seg_spend = v),
    (&["segments", "model"], |c, v| c.seg_model = v),
    (&["system", "enabled"], |c, v| c.sys_enabled = v),
];

const NUMBERS: [(Path, fn(&mut Config, f64)); 10] = [
    (&["bar_width"], bar_width),
    (&["gradient", "5h"], |c, v| c.grad_5h = unit(v)),
    (&["gradient", "7d"], |c, v| c.grad_7d = unit(v)),
    (&["system", "gradient"], |c, v| c.grad_sys = unit(v)),
    (&["thresholds", "ctx_warn"], |c, v| c.ctx_warn = v),
    (&["thresholds", "ctx_hot"], |c, v| c.ctx_hot = v),
    (&["thresholds", "quota_warn"], |c, v| c.quota_warn = v),
    (&["thresholds", "quota_hot"], |c, v| c.quota_hot = v),
    (&["thresholds", "system_warn"], |c, v| c.sys_warn = v),
    (&["thresholds", "system_hot"], |c, v| c.sys_hot = v),
];

const TEXTS: [(Path, fn(&mut Config, &str)); 7] = [
    (&["terminal_background"], |c, v| c.light_override = background_word(v)),
    (&["glyphs", "filled"], |c, v| c.filled = printable(v)),
    (&["glyphs", "empty"], |c, v| c.empty = printable(v)),
    (&["glyphs", "half"], |c, v| c.half = printable(v)),
    (&["auto_compact_marker"], |c, v| c.marker = printable(v)),
    (&["effort_style"], |c, v| c.effort_flat = by_name(&EFFORT_NAMES, v) == Some(true)),
    (&["system", "disk_path"], disk_path),
];

pub(super) fn apply(config: &mut Config, raw: &Value) -> usize {
    let mut applied = 0;
    for (path, set) in FLAGS {
        if let Some(v) = raw.at(path).and_then(Value::bool) {
            set(config, v);
            applied += 1;
        }
    }
    for (path, set) in NUMBERS {
        if let Some(v) = raw.at(path).and_then(Value::num) {
            set(config, v);
            applied += 1;
        }
    }
    for (path, set) in TEXTS {
        if let Some(v) = raw.at(path).and_then(Value::str) {
            set(config, v);
            applied += 1;
        }
    }
    applied
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::{self, Syntax};
    use alloc::format;

    fn applied_for(path: Path, leaf: &str) -> usize {
        let mut text = String::from(leaf);
        for key in path.iter().rev() {
            text = format!("{{\"{}\":{}}}", key, text);
        }
        let raw = json::parse(&text, Syntax::Commented).unwrap();
        apply(&mut Config::defaults(&raw), &raw)
    }

    #[test]
    fn every_table_key_is_read_from_its_own_path() {
        for (path, _) in FLAGS {
            assert_eq!(applied_for(path, "true"), 1, "{:?}", path);
        }
        for (path, _) in NUMBERS {
            assert_eq!(applied_for(path, "5"), 1, "{:?}", path);
        }
        for (path, _) in TEXTS {
            assert_eq!(applied_for(path, "\"x\""), 1, "{:?}", path);
        }
    }
}
