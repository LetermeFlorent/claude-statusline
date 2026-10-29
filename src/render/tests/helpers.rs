use super::common::{line, sample};
use crate::config::Rgb;
use crate::render::{Ctx, Quota, Quotas};
use alloc::format;
use alloc::string::String;

pub(super) fn quota_only(pct: Option<f64>) -> String {
    let mut data = sample();
    data.quotas = Some(Quotas {
        five_hour: Some(Quota { pct, at: None }),
        seven_day: None,
    });
    data.sys.clear();
    data.updating = false;
    let only_5h = "{\"segments\": {\"ctx\": false, \"7d\": false, \"model\": false}}";
    line(false, Some(only_5h), &data)
}

pub(super) fn luminance(c: Rgb) -> f64 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

pub(super) fn ctx_only(pct: f64, config: &str) -> String {
    let mut data = sample();
    data.ctx = Ctx {
        size: Some(1_000_000.0),
        pct: Some(pct),
        tokens: Some((pct * 10_000.0) as u64),
    };
    data.window = None;
    data.quotas = None;
    data.sys.clear();
    data.updating = false;
    let only_ctx = "\"segments\": {\"5h\": false, \"7d\": false, \"model\": false}";
    let text = format!("{{{}, {}}}", only_ctx, config);
    line(false, Some(&text), &data)
}

pub(super) fn glyphs(rendered: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for c in rendered.chars() {
        match c {
            '[' => inside = true,
            ']' => inside = false,
            c if !inside => out.push(c),
            _ => {}
        }
    }
    out
}
