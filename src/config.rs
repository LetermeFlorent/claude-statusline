use crate::json::{self, Value};
use crate::num;
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
    // Some(true) : clair force, Some(false) : sombre force, None : detection
    pub background: Option<bool>,
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
}

pub struct Colors {
    pub light: bool,
    pub ink: Rgb,
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

// light/clair ou dark/sombre, casse ignoree
pub fn background_word(s: &str) -> Option<bool> {
    let w = s.trim().to_lowercase();
    match w.as_str() {
        "light" | "clair" => Some(true),
        "dark" | "sombre" => Some(false),
        _ => None,
    }
}

pub fn hex(v: Option<&Value>) -> Option<Rgb> {
    let s = v?.str()?.trim();
    let s = s.strip_prefix('#').unwrap_or(s);
    let d: Vec<u32> = s.chars().map(|c| c.to_digit(16)).collect::<Option<Vec<u32>>>()?;
    match d.len() {
        6 => Some([(d[0] * 16 + d[1]) as f64, (d[2] * 16 + d[3]) as f64, (d[4] * 16 + d[5]) as f64]),
        3 => Some([(d[0] * 17) as f64, (d[1] * 17) as f64, (d[2] * 17) as f64]),
        _ => None,
    }
}

fn number(v: Option<&Value>, def: f64) -> f64 {
    v.and_then(|x| x.num()).unwrap_or(def)
}

fn glyph(raw: &Value, k: &str, def: &str) -> String {
    match raw.at(&["glyphs", k]).and_then(|v| v.str()) {
        Some(s) => String::from(s),
        None => String::from(def),
    }
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
    Palette { ok: get("ok", ok), warn: get("warn", warn), hot: get("hot", hot) }
}

impl Config {
    pub fn parse(text: Option<&str>) -> Config {
        let raw = text
            .and_then(|t| json::parse(t, true))
            .filter(|v| v.is_obj())
            .unwrap_or(Value::Obj(Vec::new()));
        let background = raw.get("terminal_background").and_then(|v| v.str()).and_then(background_word);
        let width = match raw.get("bar_width").and_then(|v| v.num()) {
            Some(w) => (num::trunc(w) as i64).clamp(1, 40) as usize,
            None => 8,
        };
        let sys = raw.get("system").filter(|v| v.is_obj());
        let sysget = |k: &str| sys.and_then(|s| s.get(k));
        let metrics = match sysget("metrics").and_then(|v| v.arr()) {
            Some(a) if !a.is_empty() => a
                .iter()
                .filter_map(|m| match m.str() {
                    Some("ram") => Some(Metric::Ram),
                    Some("io") => Some(Metric::Io),
                    Some("cpu") => Some(Metric::Cpu),
                    _ => None,
                })
                .collect(),
            _ => alloc::vec![Metric::Ram, Metric::Io, Metric::Cpu],
        };
        let disk_path = match sysget("disk_path").and_then(|v| v.str()) {
            Some(s) if !s.is_empty() => String::from(s),
            _ => String::from(DEFAULT_DISK),
        };
        Config {
            background,
            width,
            bold: raw.get("bold").and_then(|v| v.bool()) != Some(false),
            filled: glyph(&raw, "filled", "\u{25A0}"),
            empty: glyph(&raw, "empty", "\u{25A0}"),
            half: glyph(&raw, "half", "\u{25AA}"),
            marker: String::from(raw.get("auto_compact_marker").and_then(|v| v.str()).unwrap_or("\u{AA}")),
            effort_flat: raw.get("effort_style").and_then(|v| v.str()) == Some("flat"),
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
            pal_5h: palette(&raw, &["palettes", "5h"], [120.0, 110.0, 200.0], [170.0, 90.0, 175.0], [200.0, 60.0, 95.0]),
            pal_7d: palette(&raw, &["palettes", "7d"], [83.0, 137.0, 119.0], [184.0, 129.0, 67.0], [185.0, 85.0, 85.0]),
            pal_ram: palette(&raw, &["system", "colors", "ram"], [30.0, 120.0, 125.0], WARN_SYS, HOT_SYS),
            pal_dsk: palette(&raw, &["system", "colors", "dsk"], [105.0, 120.0, 85.0], WARN_SYS, HOT_SYS),
            pal_cpu: palette(&raw, &["system", "colors", "cpu"], [55.0, 90.0, 140.0], WARN_SYS, HOT_SYS),
            sys_enabled: sysget("enabled").and_then(|v| v.bool()) != Some(false),
            metrics,
            disk_path,
            raw,
        }
    }

    pub fn colors(&self, light: bool) -> Colors {
        let t = if light { "light" } else { "dark" };
        let c = |k: &str, def: Rgb| {
            let key = alloc::format!("{}_{}", k, t);
            hex(self.raw.at(&["colors", &key])).unwrap_or(def)
        };
        if light {
            Colors {
                light,
                ink: c("ink", [0.0, 0.0, 0.0]),
                empty: c("empty", [190.0, 180.0, 165.0]),
                ctx_from: c("ctx_from", [125.0, 185.0, 255.0]),
                ctx_to: c("ctx_to", [40.0, 90.0, 180.0]),
            }
        } else {
            Colors {
                light,
                ink: c("ink", [232.0, 234.0, 237.0]),
                empty: c("empty", [226.0, 229.0, 234.0]),
                ctx_from: c("ctx_from", [150.0, 205.0, 255.0]),
                ctx_to: c("ctx_to", [70.0, 130.0, 220.0]),
            }
        }
    }

    // Lettre du lecteur sous Windows ("C:"), "dsk" ailleurs
    pub fn disk_label(&self) -> String {
        if cfg!(windows) {
            let mut s: String = self.disk_path.chars().take(1).flat_map(|c| c.to_uppercase()).collect();
            s.push(':');
            s
        } else {
            String::from("dsk")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valeurs() {
        let c = Config::parse(Some("{\"bar_width\": 2.7, \"bold\": 0, \"glyphs\": {\"empty\": \"\"}, // x\n \"system\": {\"metrics\": [\"io\", \"RAM\", \"io\"]},}"));
        assert_eq!(c.width, 2);
        assert!(c.bold);
        assert_eq!(c.empty, "");
        assert_eq!(c.metrics, [Metric::Io, Metric::Io]);
        assert_eq!(Config::parse(Some("{\"bar_width\": 1000}")).width, 40);
        assert_eq!(Config::parse(Some("{\"bar_width\": -3}")).width, 1);
        assert_eq!(Config::parse(Some("[1]")).width, 8);
        assert_eq!(hex(Some(&Value::Str(String::from(" #abc ")))), Some([170.0, 187.0, 204.0]));
        assert_eq!(hex(Some(&Value::Str(String::from("#GG0000")))), None);
    }
}
