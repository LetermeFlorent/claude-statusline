use crate::config::hex_text;
use crate::theme::{luminance_light, Mode};

pub fn colorfgbg(v: &str) -> Option<bool> {
    let last = v.rsplit(';').next()?.trim();
    let n: u32 = last.parse().ok()?;
    Some(n == 7 || n >= 9)
}

pub fn shade(color: [f64; 3]) -> Mode {
    if luminance_light(color) {
        Mode::Light
    } else {
        Mode::Dark
    }
}

pub fn color_mode(text: &str) -> Option<Mode> {
    let text = text.trim();
    let digits = text.strip_prefix('#').unwrap_or(text);
    let opaque = match digits.len() {
        8 => digits.get(..6),
        4 => digits.get(..3),
        _ => Some(digits),
    };
    hex_text(opaque?).map(shade)
}
