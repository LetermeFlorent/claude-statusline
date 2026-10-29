use crate::json::{self, Syntax, Value};
use crate::num;
use crate::render::printable;
use alloc::string::String;
use alloc::vec::Vec;

pub type Rgb = [f64; 3];

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Metric {
    Ram,
    Io,
    Cpu,
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub ok: Rgb,
    pub warn: Rgb,
    pub hot: Rgb,
}

pub struct Config {
    raw: Value,
    pub light_override: Option<bool>,
    pub width: usize,
    pub bold: bool,
    pub filled: String,
    pub empty: String,
    pub half: String,
    pub marker: String,
    pub effort_flat: bool,
    pub seg_ctx: bool,
    pub seg_5h: bool,
    pub seg_7d: bool,
    pub seg_model: bool,
    pub grad_5h: f64,
    pub grad_7d: f64,
    pub grad_sys: f64,
    pub quota_warn: f64,
    pub quota_hot: f64,
    pub sys_warn: f64,
    pub sys_hot: f64,
    pub pal_5h: Palette,
    pub pal_7d: Palette,
    pub pal_ram: Palette,
    pub pal_dsk: Palette,
    pub pal_cpu: Palette,
    pub sys_enabled: bool,
    pub metrics: Vec<Metric>,
    pub disk_path: String,
    pub auto_update: bool,
}

pub struct Colors {
    pub light: bool,
    pub ink: Option<Rgb>,
    pub empty: Rgb,
    pub ctx_from: Rgb,
    pub ctx_to: Rgb,
}

const WARN_SYS: Rgb = [190.0, 130.0, 60.0];
const HOT_SYS: Rgb = [200.0, 60.0, 60.0];

#[cfg(windows)]
pub const DEFAULT_DISK: &str = "C:\\";
#[cfg(not(windows))]
pub const DEFAULT_DISK: &str = "/";

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

fn listed_metrics(list: Option<&Value>) -> Vec<Metric> {
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

pub fn hex(v: Option<&Value>) -> Option<Rgb> {
    hex_text(v?.str()?)
}

pub fn hex_text(s: &str) -> Option<Rgb> {
    let s = s.trim();
    let s = s.strip_prefix('#').unwrap_or(s);
    let digits: Vec<u32> = s
        .chars()
        .map(|c| c.to_digit(16))
        .collect::<Option<Vec<u32>>>()?;
    match digits.len() {
        6 => Some([
            (digits[0] * 16 + digits[1]) as f64,
            (digits[2] * 16 + digits[3]) as f64,
            (digits[4] * 16 + digits[5]) as f64,
        ]),
        3 => Some([
            (digits[0] * 17) as f64,
            (digits[1] * 17) as f64,
            (digits[2] * 17) as f64,
        ]),
        _ => None,
    }
}

fn number(v: Option<&Value>, def: f64) -> f64 {
    v.and_then(|x| x.num()).unwrap_or(def)
}

fn glyph(raw: &Value, k: &str, def: &str) -> String {
    printable(raw.at(&["glyphs", k]).and_then(|v| v.str()).unwrap_or(def))
}

fn shown(raw: &Value, k: &str) -> bool {
    raw.at(&["segments", k]).and_then(|v| v.bool()) != Some(false)
}

fn palette(raw: &Value, path: &[&str], ok: Rgb, warn: Rgb, hot: Rgb) -> Palette {
    let get = |k: &str, def: Rgb| {
        let mut p: Vec<&str> = path.to_vec();
        p.push(k);
        hex(raw.at(&p)).unwrap_or(def)
    };
    Palette {
        ok: get("ok", ok),
        warn: get("warn", warn),
        hot: get("hot", hot),
    }
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
                    .unwrap_or("\u{AA}"),
            ),
            effort_flat: raw
                .get("effort_style")
                .and_then(Value::str)
                .and_then(|s| by_name(&EFFORT_NAMES, s))
                == Some(true),
            seg_ctx: shown(&raw, "ctx"),
            seg_5h: shown(&raw, "5h"),
            seg_7d: shown(&raw, "7d"),
            seg_model: shown(&raw, "model"),
            grad_5h: num::clamp(number(raw.at(&["gradient", "5h"]), 0.55), 0.0, 1.0),
            grad_7d: num::clamp(number(raw.at(&["gradient", "7d"]), 0.3), 0.0, 1.0),
            grad_sys: num::clamp(number(sysget("gradient"), 0.45), 0.0, 1.0),
            quota_warn: number(raw.at(&["thresholds", "quota_warn"]), 30.0),
            quota_hot: number(raw.at(&["thresholds", "quota_hot"]), 70.0),
            sys_warn: number(raw.at(&["thresholds", "system_warn"]), 70.0),
            sys_hot: number(raw.at(&["thresholds", "system_hot"]), 85.0),
            pal_5h: palette(
                &raw,
                &["palettes", "5h"],
                [120.0, 110.0, 200.0],
                [170.0, 90.0, 175.0],
                [200.0, 60.0, 95.0],
            ),
            pal_7d: palette(
                &raw,
                &["palettes", "7d"],
                [83.0, 137.0, 119.0],
                [184.0, 129.0, 67.0],
                [185.0, 85.0, 85.0],
            ),
            pal_ram: palette(
                &raw,
                &["system", "colors", "ram"],
                [30.0, 120.0, 125.0],
                WARN_SYS,
                HOT_SYS,
            ),
            pal_dsk: palette(
                &raw,
                &["system", "colors", "dsk"],
                [105.0, 120.0, 85.0],
                WARN_SYS,
                HOT_SYS,
            ),
            pal_cpu: palette(
                &raw,
                &["system", "colors", "cpu"],
                [55.0, 90.0, 140.0],
                WARN_SYS,
                HOT_SYS,
            ),
            sys_enabled: sysget("enabled").and_then(|v| v.bool()) != Some(false),
            metrics: listed_metrics(sysget("metrics")),
            disk_path,
            auto_update: raw.get("auto_update").and_then(|v| v.bool()) == Some(true),
            raw,
        }
    }

    pub fn colors(&self, light: bool) -> Colors {
        let t = if light { "light" } else { "dark" };
        let custom = |k: &str| {
            let key = alloc::format!("{}_{}", k, t);
            hex(self.raw.at(&["colors", &key]))
        };
        let color = |k: &str, def: Rgb| custom(k).unwrap_or(def);
        if light {
            Colors {
                light,
                ink: custom("ink"),
                empty: color("empty", [190.0, 180.0, 165.0]),
                ctx_from: color("ctx_from", [125.0, 185.0, 255.0]),
                ctx_to: color("ctx_to", [40.0, 90.0, 180.0]),
            }
        } else {
            Colors {
                light,
                ink: custom("ink"),
                empty: color("empty", [226.0, 229.0, 234.0]),
                ctx_from: color("ctx_from", [150.0, 205.0, 255.0]),
                ctx_to: color("ctx_to", [70.0, 130.0, 220.0]),
            }
        }
    }

    pub fn disk_label(&self) -> String {
        if cfg!(windows) {
            let mut label: String = self
                .disk_path
                .chars()
                .take(1)
                .flat_map(|c| c.to_uppercase())
                .collect();
            label.push(':');
            label
        } else {
            String::from("dsk")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_are_clamped_and_bad_ones_ignored() {
        let c = Config::parse(Some("{\"bar_width\": 2.7, \"bold\": 0, \"glyphs\": {\"empty\": \"\"}, // x\n \"system\": {\"metrics\": [\"io\", \"RAM\", \"io\"]},}"));
        assert_eq!(c.width, 2);
        assert!(c.bold);
        assert_eq!(c.empty, "");
        assert_eq!(c.metrics, [Metric::Io, Metric::Ram]);
        assert_eq!(Config::parse(Some("{\"bar_width\": 1000}")).width, 40);
        assert_eq!(Config::parse(Some("{\"bar_width\": -3}")).width, 1);
        assert_eq!(Config::parse(Some("[1]")).width, 8);
        assert_eq!(
            hex(Some(&Value::Str(String::from(" #abc ")))),
            Some([170.0, 187.0, 204.0])
        );
        assert_eq!(hex(Some(&Value::Str(String::from("#GG0000")))), None);
    }

    fn metrics_of(list: &str) -> Vec<Metric> {
        let text = alloc::format!("{{\"system\": {{\"metrics\": {}}}}}", list);
        Config::parse(Some(&text)).metrics
    }

    #[test]
    fn metric_names_ignore_case_and_take_dsk_for_the_disk() {
        assert_eq!(metrics_of("[\"Ram\"]"), [Metric::Ram]);
        assert_eq!(
            metrics_of("[\" CPU \", \"dsk\"]"),
            [Metric::Cpu, Metric::Io]
        );
    }

    #[test]
    fn a_metric_listed_twice_shows_once() {
        assert_eq!(metrics_of("[\"ram\", \"ram\"]"), [Metric::Ram]);
        assert_eq!(
            metrics_of("[\"io\", \"cpu\", \"dsk\"]"),
            [Metric::Io, Metric::Cpu]
        );
    }

    #[test]
    fn a_list_with_no_known_name_shows_every_metric() {
        let every = [Metric::Ram, Metric::Io, Metric::Cpu];
        assert_eq!(metrics_of("[\"gpu\", \"net\"]"), every);
        assert_eq!(metrics_of("[]"), every);
        assert_eq!(metrics_of("\"ram\""), every);
    }

    #[test]
    fn effort_style_ignores_case() {
        let flat = |text: &str| Config::parse(Some(text)).effort_flat;
        assert!(flat("{\"effort_style\": \"Flat\"}"));
        assert!(!flat("{\"effort_style\": \"gradient\"}"));
    }

    #[test]
    fn automatic_updates_stay_off_unless_turned_on() {
        let auto_update = |text: &str| Config::parse(Some(text)).auto_update;
        assert!(!Config::parse(None).auto_update);
        assert!(!auto_update("{}"));
        assert!(!auto_update("{\"auto_update\": false}"));
        assert!(!auto_update("{\"auto_update\": \"yes\"}"));
        assert!(auto_update("{\"auto_update\": true}"));
    }

    #[test]
    fn glyphs_lose_terminal_controls() {
        let config = Config::parse(Some(
            r#"{"glyphs": {"filled": "\u001b[5m#", "half": "\u202e+", "empty": "\u0085."}, "auto_compact_marker": "\u0007*"}"#,
        ));
        assert_eq!(
            [config.filled, config.half, config.empty, config.marker],
            ["[5m#", "+", ".", "*"]
        );
    }
}
