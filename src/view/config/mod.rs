use crate::json::Value;
use alloc::string::String;
use alloc::vec::Vec;

mod colors;
mod hex;
mod names;
mod palettes;
mod parse;
#[cfg(test)]
mod tests;
mod types;

pub use hex::{hex, hex_text};
pub use names::{background_word, by_name, BACKGROUND_NAMES, EFFORT_NAMES, METRIC_NAMES};
pub use palettes::{HOT_SYS, WARN_SYS};
pub use types::{Colors, Metric, Palette, Rgb};

#[cfg(windows)]
pub const DEFAULT_DISK: &str = "C:\\";
#[cfg(not(windows))]
pub const DEFAULT_DISK: &str = "/";

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
    pub seg_spend: bool,
    pub cold_cache_marker: bool,
    pub seg_model: bool,
    pub grad_5h: f64,
    pub grad_7d: f64,
    pub grad_sys: f64,
    pub ctx_warn: f64,
    pub ctx_hot: f64,
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

impl Config {
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
