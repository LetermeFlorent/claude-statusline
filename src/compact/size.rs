use crate::json::Value;
use crate::num;

pub const MIN_WINDOW: u64 = 100_000;
pub const MAX_WINDOW: u64 = 1_000_000;
const KILO: u64 = 1_000;
const MEGA: u64 = 1_000_000;

pub fn parse(text: &str) -> Option<u64> {
    let text = text.trim();
    let (digits, unit) = if let Some(digits) = text.strip_suffix(['k', 'K']) {
        (digits, KILO)
    } else if let Some(digits) = text.strip_suffix(['m', 'M']) {
        (digits, MEGA)
    } else {
        (text, 1)
    };
    let count = digits.trim().parse::<u64>().ok().filter(|n| *n > 0)?;
    Some(count.saturating_mul(unit))
}

pub fn clamp(window: u64) -> u64 {
    window.clamp(MIN_WINDOW, MAX_WINDOW)
}

pub fn from_value(value: &Value) -> Option<u64> {
    match value {
        Value::Num(n, _) if *n > 0.0 && num::is_integral(*n) => Some(*n as u64),
        Value::Str(text) => parse(text),
        _ => None,
    }
}
