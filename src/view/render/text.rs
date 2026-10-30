use crate::messages;
use crate::num;
use alloc::format;
use alloc::string::String;
use core::ops::RangeInclusive;

const MAX_TOKENS: u64 = 999_000_000_000;
const LARGEST_TOKEN_UNIT: f64 = 1e9;
const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;
const MAX_RESET_DAYS: i64 = 99;
const MAX_RESET_SECS: f64 = (MAX_RESET_DAYS * DAY) as f64;
pub const MAX_LABEL_CHARS: usize = 48;
const BIDI_CONTROLS: [RangeInclusive<u32>; 2] = [0x202A..=0x202E, 0x2066..=0x2069];

pub fn pct_text(p: Option<f64>) -> String {
    match p {
        Some(p) => format!("{}%", num::round(p) as u64),
        None => String::from("--%"),
    }
}

pub(super) fn decimal(value: f64) -> String {
    format!("{:.1}", value).replace('.', messages::DECIMAL_POINT.text())
}

pub fn tokens(count: u64) -> String {
    let count = count.min(MAX_TOKENS);
    if count < 1000 {
        return format!("{}", count);
    }
    let amount = count as f64;
    let units = [
        (1e3, "k"),
        (1e6, "M"),
        (LARGEST_TOKEN_UNIT, messages::UNIT_BILLION.text()),
    ];
    for (size, unit) in units {
        let rounded = num::round(amount / size) as u64;
        if rounded < 1000 || size == LARGEST_TOKEN_UNIT {
            return format!("{}{}", rounded, unit);
        }
    }
    format!("{}", count)
}

pub fn reset_text(secs: f64, long: bool) -> String {
    let secs = num::floor(num::clamp(secs, 0.0, MAX_RESET_SECS)) as i64;
    if secs <= 0 {
        return messages::RESET_MINUTES.with(&["0"]);
    }
    if secs < HOUR {
        return messages::RESET_MINUTES.with(&[&format!("{}", secs / MINUTE)]);
    }
    if long && secs >= DAY {
        let (days, hours) = (secs / DAY, (secs % DAY) / HOUR);
        return messages::RESET_DAYS.with(&[&format!("{}", days), &format!("{:02}", hours)]);
    }
    let (hours, minutes) = (secs / HOUR, (secs % HOUR) / MINUTE);
    messages::RESET_HOURS.with(&[&format!("{}", hours), &format!("{:02}", minutes)])
}

fn is_printable(c: char) -> bool {
    !c.is_control()
        && !BIDI_CONTROLS
            .iter()
            .any(|range| range.contains(&(c as u32)))
}

pub fn printable(text: &str) -> String {
    text.chars().filter(|&c| is_printable(c)).collect()
}

pub fn label_text(text: &str) -> String {
    let capped: String = printable(text)
        .trim()
        .chars()
        .take(MAX_LABEL_CHARS)
        .collect();
    String::from(capped.trim_end())
}
