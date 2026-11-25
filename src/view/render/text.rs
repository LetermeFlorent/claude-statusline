use crate::messages;
use crate::num;
use alloc::format;
use alloc::string::String;
use core::ops::RangeInclusive;

const MAX_TOKENS: u64 = 999_000_000_000;
const UNIT_STEP: u64 = 1000;
const THOUSAND: f64 = UNIT_STEP as f64;
const MILLION: f64 = THOUSAND * THOUSAND;
const LARGEST_TOKEN_UNIT: f64 = MILLION * THOUSAND;
pub const PERCENT_SCALE: f64 = 100.0;
const MAX_PERCENT: f64 = 999.9;
const TENTHS: f64 = 10.0;
const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;
const MAX_RESET_DAYS: i64 = 99;
const MAX_RESET_SECS: f64 = (MAX_RESET_DAYS * DAY) as f64;
pub const MAX_LABEL_CHARS: usize = 48;
const BIDI_CONTROLS: [RangeInclusive<u32>; 2] = [0x202A..=0x202E, 0x2066..=0x2069];

pub fn tenth(p: f64) -> f64 {
    num::round(num::clamp(p, 0.0, MAX_PERCENT) * TENTHS) / TENTHS
}

pub fn pct_text(p: Option<f64>) -> String {
    match p {
        Some(p) if p.is_finite() => format!("{}%", decimal(tenth(p))),
        _ => String::from("--%"),
    }
}

pub(super) fn decimal(value: f64) -> String {
    format!("{:.1}", value).replace('.', messages::DECIMAL_POINT.text())
}

pub fn tokens(count: u64) -> String {
    let count = count.min(MAX_TOKENS);
    if count < UNIT_STEP {
        return format!("{}", count);
    }
    let amount = count as f64;
    let units = [
        (THOUSAND, "k"),
        (MILLION, "M"),
        (LARGEST_TOKEN_UNIT, messages::UNIT_BILLION.text()),
    ];
    for (size, unit) in units {
        let rounded = num::round(amount / size) as u64;
        if rounded < UNIT_STEP || size == LARGEST_TOKEN_UNIT {
            return format!("{}{}", rounded, unit);
        }
    }
    format!("{}", count)
}

fn two_digits(value: i64) -> String {
    format!("{:02}", value)
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
        return messages::RESET_DAYS.with(&[&format!("{}", days), &two_digits(hours)]);
    }
    let (hours, minutes) = (secs / HOUR, (secs % HOUR) / MINUTE);
    messages::RESET_HOURS.with(&[&format!("{}", hours), &two_digits(minutes)])
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
