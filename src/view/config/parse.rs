use super::fields::apply;
use super::names::listed_metrics;
use super::palettes::{palette, PAL_5H, PAL_7D, PAL_CPU, PAL_DSK, PAL_RAM};
use super::{Config, DEFAULT_DISK};
use crate::json::{self, Syntax, Value};
use alloc::string::String;
use alloc::vec::Vec;

impl Config {
    pub(super) fn defaults(raw: &Value) -> Config {
        Config {
            raw: Value::Obj(Vec::new()),
            light_override: None,
            width: 8,
            bold: true,
            filled: String::from("\u{25A0}"),
            empty: String::from("\u{25A0}"),
            half: String::from("\u{25AA}"),
            marker: String::from("~"),
            effort_flat: false,
            seg_ctx: true,
            seg_5h: true,
            seg_7d: true,
            seg_spend: true,
            cold_cache_marker: false,
            seg_model: true,
            grad_5h: 0.55,
            grad_7d: 0.3,
            grad_sys: 0.45,
            ctx_warn: 70.0,
            ctx_hot: 90.0,
            quota_warn: 30.0,
            quota_hot: 70.0,
            sys_warn: 70.0,
            sys_hot: 85.0,
            pal_5h: palette(raw, &PAL_5H),
            pal_7d: palette(raw, &PAL_7D),
            pal_ram: palette(raw, &PAL_RAM),
            pal_dsk: palette(raw, &PAL_DSK),
            pal_cpu: palette(raw, &PAL_CPU),
            sys_enabled: true,
            metrics: listed_metrics(raw.at(&["system", "metrics"])),
            disk_path: String::from(DEFAULT_DISK),
            auto_update: raw.get("auto_update").and_then(|v| v.bool()) == Some(true),
        }
    }

    pub fn parse(text: Option<&str>) -> Config {
        let raw = text
            .and_then(|t| json::parse(t, Syntax::Commented))
            .filter(|v| v.is_obj())
            .unwrap_or(Value::Obj(Vec::new()));
        let mut config = Config::defaults(&raw);
        apply(&mut config, &raw);
        config.raw = raw;
        config
    }
}
