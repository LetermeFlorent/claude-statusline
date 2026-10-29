use super::names::{background_word, by_name, listed_metrics, EFFORT_NAMES};
use super::palettes::{palette, PAL_5H, PAL_7D, PAL_CPU, PAL_DSK, PAL_RAM};
use super::{Config, DEFAULT_DISK};
use crate::json::{self, Syntax, Value};
use crate::num;
use crate::render::printable;
use alloc::string::String;
use alloc::vec::Vec;

fn number(v: Option<&Value>, def: f64) -> f64 {
    v.and_then(|x| x.num()).unwrap_or(def)
}

fn glyph(raw: &Value, k: &str, def: &str) -> String {
    printable(raw.at(&["glyphs", k]).and_then(|v| v.str()).unwrap_or(def))
}

fn shown(raw: &Value, k: &str) -> bool {
    raw.at(&["segments", k]).and_then(|v| v.bool()) != Some(false)
}

fn threshold(raw: &Value, key: &str, def: f64) -> f64 {
    number(raw.at(&["thresholds", key]), def)
}

fn gradient(raw: &Value, key: &str, def: f64) -> f64 {
    num::clamp(number(raw.at(&["gradient", key]), def), 0.0, 1.0)
}

impl Config {
    pub fn parse(text: Option<&str>) -> Config {
        let raw = text
            .and_then(|t| json::parse(t, Syntax::Commented))
            .filter(|v| v.is_obj())
            .unwrap_or(Value::Obj(Vec::new()));
        let light_override = raw
            .get("terminal_background")
            .and_then(|v| v.str())
            .and_then(background_word);
        let width = match raw.get("bar_width").and_then(|v| v.num()) {
            Some(w) => (num::trunc(w) as i64).clamp(1, 40) as usize,
            None => 8,
        };
        let sys = raw.get("system").filter(|v| v.is_obj());
        let sysget = |k: &str| sys.and_then(|s| s.get(k));
        let disk_path = match sysget("disk_path").and_then(|v| v.str()) {
            Some(s) if !s.is_empty() => String::from(s),
            _ => String::from(DEFAULT_DISK),
        };
        Config {
            light_override,
            width,
            bold: raw.get("bold").and_then(|v| v.bool()) != Some(false),
            filled: glyph(&raw, "filled", "\u{25A0}"),
            empty: glyph(&raw, "empty", "\u{25A0}"),
            half: glyph(&raw, "half", "\u{25AA}"),
            marker: printable(
                raw.get("auto_compact_marker")
                    .and_then(|v| v.str())
                    .unwrap_or("~"),
            ),
            effort_flat: raw
                .get("effort_style")
                .and_then(Value::str)
                .and_then(|s| by_name(&EFFORT_NAMES, s))
                == Some(true),
            seg_ctx: shown(&raw, "ctx"),
            seg_5h: shown(&raw, "5h"),
            seg_7d: shown(&raw, "7d"),
            seg_spend: shown(&raw, "spend"),
            cold_cache_marker: raw.get("cold_cache_marker").and_then(Value::bool) == Some(true),
            seg_model: shown(&raw, "model"),
            grad_5h: gradient(&raw, "5h", 0.55),
            grad_7d: gradient(&raw, "7d", 0.3),
            grad_sys: num::clamp(number(sysget("gradient"), 0.45), 0.0, 1.0),
            ctx_warn: threshold(&raw, "ctx_warn", 70.0),
            ctx_hot: threshold(&raw, "ctx_hot", 90.0),
            quota_warn: threshold(&raw, "quota_warn", 30.0),
            quota_hot: threshold(&raw, "quota_hot", 70.0),
            sys_warn: threshold(&raw, "system_warn", 70.0),
            sys_hot: threshold(&raw, "system_hot", 85.0),
            pal_5h: palette(&raw, &PAL_5H),
            pal_7d: palette(&raw, &PAL_7D),
            pal_ram: palette(&raw, &PAL_RAM),
            pal_dsk: palette(&raw, &PAL_DSK),
            pal_cpu: palette(&raw, &PAL_CPU),
            sys_enabled: sysget("enabled").and_then(|v| v.bool()) != Some(false),
            metrics: listed_metrics(sysget("metrics")),
            disk_path,
            auto_update: raw.get("auto_update").and_then(|v| v.bool()) == Some(true),
            raw,
        }
    }
}
