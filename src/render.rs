use crate::config::{Colors, Config, Metric, Rgb, HOT_SYS, WARN_SYS};
use crate::messages;
use crate::num;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;
use core::ops::RangeInclusive;

pub mod demo;
mod screen;
#[cfg(test)]
mod tests;

use fit::Layout;
use screen::{fill, fit, palette};
pub use screen::{Depth, Terminal};

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Quota {
    pub pct: Option<f64>,
    pub at: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quotas {
    pub five_hour: Option<Quota>,
    pub seven_day: Option<Quota>,
}

#[derive(Clone, Copy, Default)]
pub struct Ctx {
    pub size: Option<f64>,
    pub pct: Option<f64>,
    pub tokens: Option<u64>,
}

pub struct SysValue {
    pub metric: Metric,
    pub pct: Option<f64>,
    pub ram: Option<(f64, f64)>,
}

pub struct Data {
    pub ctx: Ctx,
    pub window: Option<u64>,
    pub quotas: Option<Quotas>,
    pub spend: Option<Quota>,
    pub cache_warm: Option<bool>,
    pub model: String,
    pub effort: Option<String>,
    pub sys: Vec<SysValue>,
    pub updating: bool,
}

const RESET: &str = "\x1b[0m";
const MAX_TOKENS: u64 = 999_000_000_000;
const LARGEST_TOKEN_UNIT: f64 = 1e9;
const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;
const MAX_RESET_DAYS: i64 = 99;
const MAX_RESET_SECS: f64 = (MAX_RESET_DAYS * DAY) as f64;
pub const MAX_LABEL_CHARS: usize = 48;
const BIDI_CONTROLS: [RangeInclusive<u32>; 2] = [0x202A..=0x202E, 0x2066..=0x2069];
const GIB: f64 = 1_073_741_824.0;
const MAX_CHANNEL: f64 = 255.0;
const LIGHT_GRADIENT_SHADE: f64 = 0.65;
const DARK_GRADIENT_TINT: f64 = 0.3;
const LINE_CAPACITY: usize = 2048;

type Pair = (Rgb, Rgb);

#[derive(Clone, Copy)]
enum Tone {
    Low,
    Medium,
    High,
    ExtraHigh,
    Max,
    Beyond,
}

const TONES: usize = Tone::Beyond as usize + 1;

const LIGHT_TONES: [Pair; TONES] = [
    ([70.0, 100.0, 97.0], [72.0, 92.0, 114.0]),
    ([71.0, 90.0, 115.0], [77.0, 85.0, 117.0]),
    ([75.0, 84.0, 118.0], [90.0, 81.0, 117.0]),
    ([88.0, 80.0, 118.0], [102.0, 78.0, 116.0]),
    ([102.0, 78.0, 116.0], [114.0, 80.0, 114.0]),
    ([116.0, 80.0, 117.0], [128.0, 93.0, 121.0]),
];

const DARK_TONES: [Pair; TONES] = [
    ([120.0, 172.0, 168.0], [124.0, 158.0, 196.0]),
    ([122.0, 156.0, 198.0], [132.0, 146.0, 202.0]),
    ([130.0, 144.0, 204.0], [156.0, 140.0, 202.0]),
    ([152.0, 138.0, 204.0], [176.0, 134.0, 200.0]),
    ([176.0, 134.0, 200.0], [196.0, 138.0, 196.0]),
    ([200.0, 138.0, 202.0], [220.0, 160.0, 208.0]),
];

impl Tone {
    fn of(effort: &str) -> Tone {
        match effort {
            "low" | "auto" => Tone::Low,
            "medium" => Tone::Medium,
            "high" => Tone::High,
            "xhigh" => Tone::ExtraHigh,
            "max" => Tone::Max,
            _ => Tone::Beyond,
        }
    }

    fn pair(self, light: bool) -> Pair {
        let tones = if light { &LIGHT_TONES } else { &DARK_TONES };
        tones[self as usize]
    }
}

#[derive(Clone, Copy, PartialEq)]
enum QuotaWindow {
    FiveHour,
    SevenDay,
    Spend,
}

impl QuotaWindow {
    fn label(self) -> &'static str {
        match self {
            QuotaWindow::FiveHour => "5h",
            QuotaWindow::SevenDay => messages::SEVEN_DAY_LABEL.text(),
            QuotaWindow::Spend => messages::SPEND_LABEL.text(),
        }
    }
}

fn lerp(a: Rgb, b: Rgb, t: f64) -> Rgb {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn rgb(c: Rgb) -> Rgb {
    [num::round(c[0]), num::round(c[1]), num::round(c[2])]
}

struct Out<'a> {
    line: String,
    cfg: &'a Config,
    col: &'a Colors,
    layout: Layout,
    depth: Depth,
}

impl Out<'_> {
    fn fg(&mut self, c: Rgb) {
        let _ = match self.depth {
            Depth::TrueColor => write!(
                self.line,
                "\x1b[38;2;{};{};{}m",
                c[0] as i64, c[1] as i64, c[2] as i64
            ),
            Depth::Indexed => write!(self.line, "\x1b[38;5;{}m", palette::indexed(c)),
        };
    }

    fn bold(&mut self) {
        if self.cfg.bold {
            self.line.push_str("\x1b[1m");
        }
    }

    fn tint(&mut self, c: Option<Rgb>) {
        if let Some(c) = c {
            self.fg(c);
        }
    }

    fn strong(&mut self, text: &str, c: Option<Rgb>) {
        self.bold();
        self.tint(c);
        self.line.push_str(text);
        self.line.push_str(RESET);
    }

    fn label(&mut self, text: &str) {
        let ink = self.col.ink;
        self.strong(text, ink);
    }

    fn bar(&mut self, pct: Option<f64>, color_at: &dyn Fn(usize, usize) -> Rgb) -> Option<Rgb> {
        let width = self.layout.width;
        let empty = self.col.empty;
        let Some(pct) = pct else {
            self.fg(empty);
            for _ in 0..width {
                self.line.push_str(&self.cfg.empty);
            }
            self.line.push_str(RESET);
            return None;
        };
        let fill = fill::of(pct, width);
        let filled = fill.cells;
        let mut last = None;
        for i in 0..width {
            if i < filled {
                let c = color_at(i, filled);
                last = Some(c);
                self.fg(c);
                let glyph = if fill.half && i == filled - 1 {
                    &self.cfg.half
                } else {
                    &self.cfg.filled
                };
                self.line.push_str(glyph);
            } else {
                self.fg(empty);
                self.line.push_str(&self.cfg.empty);
            }
        }
        self.line.push_str(RESET);
        last
    }

    fn gradient_bar(&mut self, pct: Option<f64>, start: Rgb, gradient: f64) -> Option<Rgb> {
        let light = self.col.light;
        let target = start.map(|channel| {
            num::round(if light {
                channel * LIGHT_GRADIENT_SHADE
            } else {
                channel + (MAX_CHANNEL - channel) * DARK_GRADIENT_TINT
            })
        });
        let end = lerp(start, target, gradient);
        self.bar(pct, &|cell, filled| {
            rgb(lerp(
                start,
                end,
                if filled > 1 {
                    cell as f64 / (filled - 1) as f64
                } else {
                    0.0
                },
            ))
        })
    }

    fn value(&mut self, text: &str, pct: Option<f64>, last: Option<Rgb>) {
        let color = match (pct, last) {
            (Some(p), Some(l)) if num::round(p) >= 100.0 => Some(l),
            _ => self.col.ink,
        };
        self.strong(text, color);
    }

    fn ctx(&mut self, data: &Data) {
        let size = data.ctx.size.filter(|s| *s > 0.0);
        let mut pct = data.ctx.pct;
        let used: Option<f64> = match (data.ctx.tokens, pct, size) {
            (Some(t), _, _) => Some(t as f64),
            (None, Some(p), Some(s)) => Some((p * s / 100.0) as u64 as f64),
            _ => None,
        };
        let mut denom = size;
        let mut marked = false;
        if let (Some(size), Some(window)) = (size, data.window) {
            if window > 0 && (window as f64) < size {
                denom = Some(window as f64);
                pct = used.map(|u| u / window as f64 * 100.0);
                marked = true;
            }
        }
        if marked && !self.cfg.marker.is_empty() {
            let marker = self.cfg.marker.clone();
            self.label(&marker);
            self.line.push(' ');
        }
        self.label("ctx");
        self.line.push(' ');
        let (from, to) = (self.col.ctx_from, self.col.ctx_to);
        let span = if self.layout.width > 1 {
            (self.layout.width - 1) as f64
        } else {
            1.0
        };
        let grad = self.cfg.grad_sys;
        let last = match pct {
            Some(p) if p >= self.cfg.ctx_hot => self.gradient_bar(pct, HOT_SYS, grad),
            Some(p) if p >= self.cfg.ctx_warn => self.gradient_bar(pct, WARN_SYS, grad),
            _ => self.bar(pct, &|i, _| rgb(lerp(from, to, i as f64 / span))),
        };
        self.line.push(' ');
        let text = match (denom, used) {
            (Some(denom), Some(used)) => {
                format!("{}/{}", tokens(used as u64), tokens(denom as u64))
            }
            _ => pct_text(pct),
        };
        self.value(&text, pct, last);
        if self.cfg.cold_cache_marker && data.cache_warm == Some(false) {
            self.line.push(' ');
            self.label(messages::COLD_CACHE_LABEL.text());
        }
    }

    fn quota(&mut self, window: QuotaWindow, quota: Option<Quota>, now: f64) {
        let (palette, gradient) = match window {
            QuotaWindow::FiveHour | QuotaWindow::Spend => (self.cfg.pal_5h, self.cfg.grad_5h),
            QuotaWindow::SevenDay => (self.cfg.pal_7d, self.cfg.grad_7d),
        };
        let quota = quota.unwrap_or_default();
        self.label(window.label());
        self.line.push(' ');
        let start = match quota.pct {
            Some(p) if p >= self.cfg.quota_hot => palette.hot,
            Some(p) if p >= self.cfg.quota_warn => palette.warn,
            _ => palette.ok,
        };
        let last = self.gradient_bar(quota.pct, start, gradient);
        self.line.push(' ');
        self.value(&pct_text(quota.pct), quota.pct, last);
        if !self.layout.resets {
            return;
        }
        if let Some(at) = quota.at {
            self.line.push(' ');
            self.label(&reset_text(at - now, window != QuotaWindow::FiveHour));
        } else if window == QuotaWindow::FiveHour {
            self.line.push(' ');
            self.label("--");
        }
    }

    fn model(&mut self, data: &Data) {
        self.line.push_str("\x1b[4m");
        let ink = self.col.ink;
        self.tint(ink);
        self.line.push_str(&data.model);
        self.line.push_str(RESET);
        let Some(level) = data.effort.as_deref().filter(|_| self.layout.effort) else {
            return;
        };
        let key = level.to_lowercase();
        let word = if key == "ultracode" {
            format!("{}(+workflows)", level)
        } else {
            String::from(level)
        };
        let (start, end) = Tone::of(&key).pair(self.col.light);
        self.label(":");
        if self.cfg.effort_flat {
            self.strong(&word, Some(end));
            return;
        }
        let chars: Vec<char> = word.chars().collect();
        for (i, letter) in chars.iter().enumerate() {
            let progress = if chars.len() > 1 {
                i as f64 / (chars.len() - 1) as f64
            } else {
                0.0
            };
            self.bold();
            self.fg(rgb(lerp(start, end, progress)));
            self.line.push(*letter);
        }
        self.line.push_str(RESET);
    }

    fn system(&mut self, reading: &SysValue) {
        let cfg = self.cfg;
        let (name, palette, pct, text) = match reading.metric {
            Metric::Ram => {
                let pct = reading
                    .ram
                    .filter(|(total, _)| *total > 0.0)
                    .map(|(total, available)| (total - available) / total * 100.0);
                let text = match (pct, reading.ram) {
                    (Some(_), Some((total, available))) => messages::RAM_USAGE
                        .with(&[&decimal((total - available) / GIB), &decimal(total / GIB)]),
                    _ => String::from("--%"),
                };
                (String::from("ram"), cfg.pal_ram, pct, text)
            }
            Metric::Io => (
                cfg.disk_label(),
                cfg.pal_dsk,
                reading.pct,
                pct_text(reading.pct),
            ),
            Metric::Cpu => (
                String::from("cpu"),
                cfg.pal_cpu,
                reading.pct,
                pct_text(reading.pct),
            ),
        };
        self.label(&name);
        self.line.push(' ');
        let start = match pct {
            Some(p) if p > cfg.sys_hot => palette.hot,
            Some(p) if p > cfg.sys_warn => palette.warn,
            _ => palette.ok,
        };
        let last = self.gradient_bar(pct, start, cfg.grad_sys);
        self.line.push(' ');
        self.value(&text, pct, last);
    }

    fn sep(&mut self, first: &mut bool) {
        if !*first {
            self.line.push(' ');
            self.label("|");
            self.line.push(' ');
        }
        *first = false;
    }
}

pub fn pct_text(p: Option<f64>) -> String {
    match p {
        Some(p) => format!("{}%", num::round(p) as u64),
        None => String::from("--%"),
    }
}

fn decimal(value: f64) -> String {
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

pub fn render(cfg: &Config, col: &Colors, data: &Data, now: f64, term: &Terminal) -> String {
    let mut layout = Layout::full(cfg.width);
    loop {
        let text = draw(cfg, col, data, now, layout, term.depth);
        let fits = term
            .columns
            .is_none_or(|columns| fit::widest(&text) <= columns);
        match layout.narrower() {
            Some(narrower) if !fits => layout = narrower,
            _ => return text,
        }
    }
}

fn has_windows(data: &Data) -> bool {
    data.quotas
        .is_some_and(|q| q.five_hour.is_some() || q.seven_day.is_some())
}

fn draw(cfg: &Config, col: &Colors, data: &Data, now: f64, layout: Layout, depth: Depth) -> String {
    let mut out = Out {
        line: String::with_capacity(LINE_CAPACITY),
        cfg,
        col,
        layout,
        depth,
    };
    let mut first = true;
    if cfg.seg_ctx {
        out.sep(&mut first);
        out.ctx(data);
    }
    if let Some(quotas) = data.quotas {
        if cfg.seg_5h {
            out.sep(&mut first);
            out.quota(QuotaWindow::FiveHour, quotas.five_hour, now);
        }
        if cfg.seg_7d {
            out.sep(&mut first);
            out.quota(QuotaWindow::SevenDay, quotas.seven_day, now);
        }
    }
    if let Some(spend) = data.spend.filter(|_| cfg.seg_spend && !has_windows(data)) {
        out.sep(&mut first);
        out.quota(QuotaWindow::Spend, Some(spend), now);
    }
    if cfg.seg_model {
        out.sep(&mut first);
        out.model(data);
    }
    if !data.sys.is_empty() || data.updating {
        out.line.push('\n');
        first = true;
        for reading in &data.sys {
            out.sep(&mut first);
            out.system(reading);
        }
        if data.updating {
            out.sep(&mut first);
            out.label(messages::UPDATE_LABEL.text());
        }
    }
    out.line.push_str(RESET);
    out.line.push('\n');
    out.line
}
