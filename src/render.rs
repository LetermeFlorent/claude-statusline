use crate::config::{Colors, Config, Metric, Rgb, HOT_SYS, WARN_SYS};
use crate::num;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;
use core::ops::RangeInclusive;

mod fill;
mod fit;
mod palette;
mod terminal;

use fit::Layout;
pub use terminal::{Depth, Terminal};

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
    pub model: String,
    pub effort: Option<String>,
    pub sys: Vec<SysValue>,
    pub updating: bool,
}

const RESET: &str = "\x1b[0m";
const MAX_TOKENS: u64 = 999_000_000_000;
const LARGEST_TOKEN_UNIT: f64 = 1e9;
const TOKEN_UNITS: [(f64, char); 3] = [(1e3, 'k'), (1e6, 'M'), (LARGEST_TOKEN_UNIT, 'B')];
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
}

impl QuotaWindow {
    fn label(self) -> &'static str {
        match self {
            QuotaWindow::FiveHour => "5h",
            QuotaWindow::SevenDay => "7d",
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
    }

    fn quota(&mut self, window: QuotaWindow, quota: Option<Quota>, now: f64) {
        let (palette, gradient) = match window {
            QuotaWindow::FiveHour => (self.cfg.pal_5h, self.cfg.grad_5h),
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
            self.label(&reset_text(at - now, window == QuotaWindow::SevenDay));
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
                    (Some(_), Some((total, available))) => {
                        format!("{:.1}/{:.1}G", (total - available) / GIB, total / GIB)
                    }
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

pub fn tokens(count: u64) -> String {
    let count = count.min(MAX_TOKENS);
    if count < 1000 {
        return format!("{}", count);
    }
    let amount = count as f64;
    for (size, unit) in TOKEN_UNITS {
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
        return String::from("0m");
    }
    if secs < HOUR {
        return format!("{}m", secs / MINUTE);
    }
    if long && secs >= DAY {
        return format!("{}d{:02}h", secs / DAY, (secs % DAY) / HOUR);
    }
    format!("{}h{:02}m", secs / HOUR, (secs % HOUR) / MINUTE)
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
            out.label("update");
        }
    }
    out.line.push_str(RESET);
    out.line.push('\n');
    out.line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_shorten_and_cells_stay_within_the_bar() {
        assert_eq!(tokens(2500), "3k");
        assert_eq!(tokens(3500), "4k");
        assert_eq!(tokens(999), "999");
        assert_eq!(reset_text(59.9, false), "0m");
        assert_eq!(reset_text(5000.0, false), "1h23m");
        assert_eq!(reset_text(200000.0, true), "2d07h");
        assert_eq!(reset_text(-5.0, true), "0m");
    }

    #[test]
    fn huge_values_are_clamped() {
        assert_eq!(tokens(u64::MAX), "999B");
        assert_eq!(reset_text(1e300, true), "99d00h");
        assert_eq!(reset_text(f64::INFINITY, false), "2376h00m");
        assert_eq!(reset_text(f64::NAN, true), "0m");
    }

    const NOW: f64 = 1_000_000.0;

    fn readable(rendered: &str) -> String {
        let mut parts = rendered.split('\x1b');
        let mut out = String::from(parts.next().unwrap_or(""));
        for part in parts {
            let (code, text) = part[1..].split_once('m').unwrap();
            match code {
                "0" => out.push_str("[/]"),
                "1" => out.push_str("[b]"),
                "2" => out.push_str("[dim]"),
                "4" => out.push_str("[u]"),
                _ => {
                    let channels: Vec<u8> = code
                        .strip_prefix("38;2;")
                        .unwrap()
                        .split(';')
                        .map(|x| x.parse().unwrap())
                        .collect();
                    let _ = write!(
                        out,
                        "[#{:02x}{:02x}{:02x}]",
                        channels[0], channels[1], channels[2]
                    );
                }
            }
            out.push_str(text);
        }
        out
    }

    fn sample() -> Data {
        Data {
            ctx: Ctx {
                size: Some(1_000_000.0),
                pct: Some(6.0),
                tokens: Some(60_000),
            },
            window: Some(200_000),
            quotas: Some(Quotas {
                five_hour: Some(Quota {
                    pct: Some(36.0),
                    at: Some(NOW + 3600.0),
                }),
                seven_day: Some(Quota {
                    pct: Some(81.0),
                    at: Some(NOW + 200_000.0),
                }),
            }),
            model: String::from("Opus 4"),
            effort: Some(String::from("high")),
            sys: alloc::vec![
                SysValue {
                    metric: Metric::Ram,
                    pct: None,
                    ram: Some((16.0 * GIB, 4.0 * GIB)),
                },
                SysValue {
                    metric: Metric::Io,
                    pct: Some(90.0),
                    ram: None,
                },
                SysValue {
                    metric: Metric::Cpu,
                    pct: Some(12.0),
                    ram: None,
                },
            ],
            updating: true,
        }
    }

    fn line(light: bool, config: Option<&str>, data: &Data) -> String {
        let cfg = Config::parse(config);
        readable(&render(
            &cfg,
            &cfg.colors(light),
            data,
            NOW,
            &Terminal::WIDE,
        ))
    }

    fn fitted(columns: usize, config: Option<&str>) -> String {
        let cfg = Config::parse(config);
        let term = Terminal {
            columns: Some(columns),
            depth: Depth::TrueColor,
        };
        readable(&render(&cfg, &cfg.colors(false), &sample(), NOW, &term))
    }

    fn quota_only(pct: Option<f64>) -> String {
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

    const DISK: &str = if cfg!(windows) { "C:" } else { "dsk" };

    #[test]
    fn dark_line() {
        let expected = format!(
            concat!(
                "[b]~[/] [b]ctx[/] [#96cdff]■[#8bc2fa]■[#7fb8f5]▪[#55595f]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[/] [b]60k/200k[/]",
                " [b]|[/] [b]5h[/] [#aa5aaf]■[#b168b6]■[#b876bc]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[/] [b]36%[/] [b]1h00m[/]",
                " [b]|[/] [b]7d[/] [#b95555]■[#ba5858]■[#bb5a5a]■[#bc5d5d]■[#bd5f5f]■[#be6262]■[#bf6464]▪[#55595f]■[/] [b]81%[/] [b]2d07h[/]",
                " [b]|[/] [u]Opus 4[/][b]:[/][b][#8290cc]h[b][#8b8fcb]i[b][#938dcb]g[b][#9c8cca]h[/]\n",
                "[b]ram[/] [#be823c]■[#c08541]■[#c28947]■[#c38c4c]■[#c59051]■[#c79357]■[#55595f]■[#55595f]■[/] [b]12.0/16.0G[/]",
                " [b]|[/] [b]{disk}[/] [#c83c3c]■[#c94040]■[#cb4545]■[#cc4949]■[#cd4e4e]■[#ce5252]■[#d05757]■[#55595f]■[/] [b]90%[/]",
                " [b]|[/] [b]cpu[/] [#4f7fc4]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[/] [b]12%[/]",
                " [b]|[/] [b]update[/][/]\n",
            ),
            disk = DISK
        );
        assert_eq!(line(false, None, &sample()), expected);
    }

    #[test]
    fn light_line() {
        let expected = format!(
            concat!(
                "[b]~[/] [b]ctx[/] [#3d8beb]■[#3a84e3]■[#377ddb]▪[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[/] [b]60k/200k[/]",
                " [b]|[/] [b]5h[/] [#aa5aaf]■[#9a519e]■[#8a498d]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[/] [b]36%[/] [b]1h00m[/]",
                " [b]|[/] [b]7d[/] [#b95555]■[#b65454]■[#b35252]■[#af5151]■[#ac4f4f]■[#a94e4e]■[#a64c4c]▪[#c4c8ce]■[/] [b]81%[/] [b]2d07h[/]",
                " [b]|[/] [u]Opus 4[/][b]:[/][b][#4b5476]h[b][#505376]i[b][#555275]g[b][#5a5175]h[/]\n",
                "[b]ram[/] [#be823c]■[#b87e3a]■[#b27a38]■[#ac7636]■[#a67234]■[#a06e33]■[#c4c8ce]■[#c4c8ce]■[/] [b]12.0/16.0G[/]",
                " [b]|[/] [b]{disk}[/] [#c83c3c]■[#c33a3a]■[#be3939]■[#b83737]■[#b33636]■[#ae3434]■[#a93333]■[#c4c8ce]■[/] [b]90%[/]",
                " [b]|[/] [b]cpu[/] [#4f7fc4]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[/] [b]12%[/]",
                " [b]|[/] [b]update[/][/]\n",
            ),
            disk = DISK
        );
        assert_eq!(line(true, None, &sample()), expected);
    }

    #[test]
    fn text_keeps_the_terminal_color_without_ink() {
        for light in [false, true] {
            let text = line(light, None, &sample());
            for word in [
                "ctx", "60k/200k", "|", "5h", "36%", "1h00m", "ram", "update",
            ] {
                assert!(
                    text.contains(&format!("[b]{}[/]", word)),
                    "{} {}",
                    word,
                    text
                );
            }
            assert!(text.contains("[u]Opus 4[/][b]:[/]"), "{}", text);
        }
    }

    #[test]
    fn ink_colors_the_text_when_it_is_set() {
        let config = "{\"colors\": {\"ink_dark\": \"#112233\"}}";
        let text = line(false, Some(config), &sample());
        assert!(text.contains("[b][#112233]ctx[/]"), "{}", text);
        assert!(text.contains("[b][#112233]60k/200k[/]"), "{}", text);
        assert!(text.contains("[u][#112233]Opus 4[/]"), "{}", text);
        let light = line(true, Some(config), &sample());
        assert!(light.contains("[b]ctx[/]"), "{}", light);
    }

    #[test]
    fn quota_bar_levels() {
        let levels = [Some(0.0), Some(50.0), Some(80.0), Some(100.0), None].map(quota_only);
        assert_eq!(
            levels,
            [
                "[b]5h[/] [#55595f]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[/] [b]0%[/] [b]--[/][/]\n",
                "[b]5h[/] [#aa5aaf]■[#af63b3]■[#b46cb8]■[#b876bc]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[/] [b]50%[/] [b]--[/][/]\n",
                "[b]5h[/] [#c83c5f]■[#ca4163]■[#cb4768]■[#cd4c6c]■[#ce5271]■[#d05775]■[#d15c79]▪[#55595f]■[/] [b]80%[/] [b]--[/][/]\n",
                "[b]5h[/] [#c83c5f]■[#c94163]■[#cb4567]■[#cc4a6a]■[#cd4f6e]■[#cf5372]■[#d05876]■[#d15c79]■[/] [b][#d15c79]100%[/] [b]--[/][/]\n",
                "[b]5h[/] [#55595f]■■■■■■■■[/] [b]--%[/] [b]--[/][/]\n",
            ]
        );
    }

    fn luminance(c: Rgb) -> f64 {
        0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
    }

    #[test]
    fn empty_cells_are_dimmer_than_full_ones_on_dark_and_lighter_on_light() {
        let cfg = Config::parse(None);
        let (dark, light) = (cfg.colors(false), cfg.colors(true));
        assert_eq!(dark.empty, [85.0, 89.0, 95.0]);
        assert_eq!(light.empty, [196.0, 200.0, 206.0]);
        assert_eq!(light.ctx_from, [61.0, 139.0, 235.0]);
        assert!(luminance(dark.empty) < luminance(dark.ctx_to));
        assert!(luminance(light.empty) > luminance(light.ctx_from));
        assert!(luminance(light.empty) > luminance(light.ctx_to));
    }

    fn ctx_only(pct: f64, config: &str) -> String {
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

    fn glyphs(rendered: &str) -> String {
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

    #[test]
    fn the_small_glyph_marks_a_real_half_cell_only() {
        let dots = "\"glyphs\": {\"empty\": \".\"}";
        let bar = |pct| glyphs(&ctx_only(pct, dots));
        assert_eq!(
            bar(44.0),
            "ctx \u{25A0}\u{25A0}\u{25A0}\u{25AA}.... 440k/1M\n"
        );
        assert_eq!(
            bar(50.0),
            "ctx \u{25A0}\u{25A0}\u{25A0}\u{25A0}.... 500k/1M\n"
        );
        assert_eq!(
            bar(100.0),
            "ctx \u{25A0}\u{25A0}\u{25A0}\u{25A0}\u{25A0}\u{25A0}\u{25A0}\u{25A0} 1M/1M\n"
        );
        assert_eq!(bar(6.0), "ctx \u{25AA}....... 60k/1M\n");
    }

    #[test]
    fn the_context_bar_turns_warm_from_70_and_hot_from_90() {
        let first_cell = |pct: f64, config: &str| {
            let text = ctx_only(pct, config);
            let at = text.find("[#").unwrap();
            String::from(&text[at..at + 9])
        };
        assert_eq!(first_cell(69.0, "\"bold\": true"), "[#96cdff]");
        assert_eq!(first_cell(70.0, "\"bold\": true"), "[#be823c]");
        assert_eq!(first_cell(89.0, "\"bold\": true"), "[#be823c]");
        assert_eq!(first_cell(90.0, "\"bold\": true"), "[#c83c3c]");
        let raised = "\"thresholds\": {\"ctx_warn\": 50, \"ctx_hot\": 60}";
        assert_eq!(first_cell(49.0, raised), "[#96cdff]");
        assert_eq!(first_cell(50.0, raised), "[#be823c]");
        assert_eq!(first_cell(60.0, raised), "[#c83c3c]");
    }

    #[test]
    fn the_cpu_bar_is_light_enough_to_see_on_dark() {
        assert_eq!(Config::parse(None).pal_cpu.ok, [79.0, 127.0, 196.0]);
        let text = line(false, None, &sample());
        assert!(text.contains("cpu[/] [#4f7fc4]"), "{}", text);
    }

    #[test]
    fn the_auto_compact_marker_is_a_visible_tilde_and_a_space() {
        let text = line(false, None, &sample());
        assert!(text.starts_with("[b]~[/] [b]ctx[/]"), "{}", text);
        let old = line(
            false,
            Some("{\"auto_compact_marker\": \"\u{AA}\"}"),
            &sample(),
        );
        assert!(old.starts_with("[b]\u{AA}[/] [b]ctx[/]"), "{}", old);
        let none = line(false, Some("{\"auto_compact_marker\": \"\"}"), &sample());
        assert!(none.starts_with("[b]ctx[/]"), "{}", none);
        assert!(!text.contains("[dim]"), "{}", text);
    }

    #[test]
    fn a_count_close_to_a_million_moves_to_the_next_unit() {
        assert_eq!(tokens(999_499), "999k");
        assert_eq!(tokens(999_500), "1M");
        assert_eq!(tokens(999_999), "1M");
        assert_eq!(tokens(999_499_999), "999M");
        assert_eq!(tokens(999_500_000), "1B");
        assert_eq!(tokens(1_499), "1k");
        assert_eq!(tokens(1_500), "2k");
        assert_eq!(tokens(2_500), "3k");
    }

    #[test]
    fn the_line_fits_in_80_columns() {
        let wide = glyphs(&line(false, None, &sample()));
        assert!(fit::widest(&wide) > 80, "{}", wide);
        let narrow = glyphs(&fitted(80, None));
        assert!(fit::widest(&narrow) <= 80, "{}", narrow);
    }

    #[test]
    fn the_reset_times_go_first_then_the_effort_then_the_bar_width() {
        let text = |columns| glyphs(&fitted(columns, None));
        let bar = "\u{25A0}".repeat(8);
        assert!(text(85).contains("1h00"), "{}", text(85));
        let no_resets = text(84);
        assert!(!no_resets.contains("1h00"), "{}", no_resets);
        assert!(no_resets.contains(":high"), "{}", no_resets);
        let no_effort = text(70);
        assert!(!no_effort.contains(":high"), "{}", no_effort);
        assert!(no_effort.contains("Opus 4"), "{}", no_effort);
        assert!(no_effort.contains(&bar), "{}", no_effort);
        let narrow = text(60);
        assert!(!narrow.contains(&bar), "{}", narrow);
        assert!(fit::widest(&narrow) <= 60, "{}", narrow);
    }

    #[test]
    fn a_bar_width_of_4_fits_in_80_columns_with_every_detail() {
        let text = glyphs(&fitted(80, Some("{\"bar_width\": 4}")));
        assert!(text.contains("1h00") && text.contains(":high"), "{}", text);
        assert!(fit::widest(&text) <= 80, "{}", text);
    }

    fn raw(depth: Depth) -> String {
        let cfg = Config::parse(None);
        let term = Terminal {
            columns: None,
            depth,
        };
        render(&cfg, &cfg.colors(false), &sample(), NOW, &term)
    }

    #[test]
    fn without_truecolor_only_256_color_sequences_are_sent() {
        let indexed = raw(Depth::Indexed);
        assert!(!indexed.contains("38;2;"), "{:?}", indexed);
        assert!(indexed.contains("[38;5;240m"), "{:?}", indexed);
        let colors = indexed.matches("38;5;").count();
        assert_eq!(colors, indexed.matches("[38;").count());
        let true_color = raw(Depth::TrueColor);
        assert!(!true_color.contains("38;5;"), "{:?}", true_color);
        assert!(true_color.contains("38;2;85;89;95"), "{:?}", true_color);
    }

    #[test]
    fn negative_percent_reads_zero() {
        assert_eq!(pct_text(Some(-3.0)), "0%");
    }
}
