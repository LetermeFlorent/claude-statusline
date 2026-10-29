use super::Rgb;
use crate::json::Value;
use alloc::vec::Vec;

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
