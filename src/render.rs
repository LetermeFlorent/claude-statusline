use crate::config::{Colors, Config, Metric, Rgb};
use crate::num;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;
use core::ops::RangeInclusive;

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
    // Les deux segments de quota n'apparaissent que si l'une des fenetres est connue
    pub quotas: Option<Quotas>,
    pub model: String,
    pub effort: Option<String>,
    pub sys: Vec<SysValue>,
    pub updating: bool,
}

const RESET: &str = "\x1b[0m";
const MAX_TOKENS: u64 = 999_000_000_000;
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
}

impl Out<'_> {
    fn fg(&mut self, c: Rgb) {
        let _ = write!(
            self.line,
            "\x1b[38;2;{};{};{}m",
            c[0] as i64, c[1] as i64, c[2] as i64
        );
    }

    fn bold(&mut self) {
        if self.cfg.bold {
            self.line.push_str("\x1b[1m");
        }
    }

    fn strong(&mut self, text: &str, c: Rgb) {
        self.bold();
        self.fg(c);
        self.line.push_str(text);
        self.line.push_str(RESET);
    }

    fn label(&mut self, text: &str) {
        let ink = self.col.ink;
        self.strong(text, ink);
    }

    // Une cellule par couleur ; une barre sans valeur est un seul bloc vide
    fn bar(&mut self, pct: Option<f64>, color_at: &dyn Fn(usize, usize) -> Rgb) -> Option<Rgb> {
        let width = self.cfg.width;
        let empty = self.col.empty;
        let Some(pct) = pct else {
            self.fg(empty);
            for _ in 0..width {
                self.line.push_str(&self.cfg.empty);
            }
            self.line.push_str(RESET);
            return None;
        };
        let filled = cell_count(pct, width);
        let mut last = None;
        for i in 0..width {
            if i < filled {
                let c = color_at(i, filled);
                last = Some(c);
                self.fg(c);
                let glyph = if i == filled - 1 {
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
            (Some(p), Some(l)) if num::round(p) >= 100.0 => l,
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
        if marked {
            self.line.push_str("\x1b[2m");
            let ink = self.col.ink;
            self.fg(ink);
            self.line.push_str(&self.cfg.marker);
            self.line.push_str(RESET);
        }
        self.label("ctx");
        self.line.push(' ');
        let (from, to) = (self.col.ctx_from, self.col.ctx_to);
        let span = if self.cfg.width > 1 {
            (self.cfg.width - 1) as f64
        } else {
            1.0
        };
        let last = self.bar(pct, &|i, _| rgb(lerp(from, to, i as f64 / span)));
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
        self.fg(ink);
        self.line.push_str(&data.model);
        self.line.push_str(RESET);
        let Some(level) = data.effort.as_deref() else {
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
            self.strong(&word, end);
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

pub fn cell_count(pct: f64, width: usize) -> usize {
    if !(pct > 0.0) {
        return 0;
    }
    let half_cells = num::round(pct * 2.0 * width as f64 / 100.0);
    let cells = num::ceil(half_cells / 2.0);
    let cells = if cells < 1.0 { 1.0 } else { cells };
    if cells > width as f64 {
        width
    } else {
        cells as usize
    }
}

pub fn pct_text(p: Option<f64>) -> String {
    match p {
        // Une valeur negative s'affiche 0%
        Some(p) => format!("{}%", num::round(p) as u64),
        None => String::from("--%"),
    }
}

pub fn tokens(count: u64) -> String {
    let count = count.min(MAX_TOKENS);
    let amount = count as f64;
    if count < 1000 {
        format!("{}", count)
    } else if amount < 1e6 {
        format!("{:.0}k", amount / 1e3)
    } else if amount < 1e9 {
        format!("{:.0}M", amount / 1e6)
    } else {
        format!("{:.0}B", amount / 1e9)
    }
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

pub fn render(cfg: &Config, col: &Colors, data: &Data, now: f64) -> String {
    let mut out = Out {
        line: String::with_capacity(LINE_CAPACITY),
        cfg,
        col,
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
    fn textes() {
        assert_eq!(tokens(2500), "2k");
        assert_eq!(tokens(3500), "4k");
        assert_eq!(tokens(999), "999");
        assert_eq!(reset_text(59.9, false), "0m");
        assert_eq!(reset_text(5000.0, false), "1h23m");
        assert_eq!(reset_text(200000.0, true), "2d07h");
        assert_eq!(reset_text(-5.0, true), "0m");
        assert_eq!(cell_count(50.0, 8), 4);
        assert_eq!(cell_count(0.5, 8), 1);
        assert_eq!(cell_count(150.0, 8), 8);
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
        readable(&render(&cfg, &cfg.colors(light), data, NOW))
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
                "[dim][#e8eaed]ª[/][b][#e8eaed]ctx[/] [#96cdff]■[#8bc2fa]■[#7fb8f5]▪[#e2e5ea]■[#e2e5ea]■[#e2e5ea]■[#e2e5ea]■[#e2e5ea]■[/] [b][#e8eaed]60k/200k[/]",
                " [b][#e8eaed]|[/] [b][#e8eaed]5h[/] [#aa5aaf]■[#b168b6]■[#b876bc]▪[#e2e5ea]■[#e2e5ea]■[#e2e5ea]■[#e2e5ea]■[#e2e5ea]■[/] [b][#e8eaed]36%[/] [b][#e8eaed]1h00m[/]",
                " [b][#e8eaed]|[/] [b][#e8eaed]7d[/] [#b95555]■[#ba5858]■[#bb5a5a]■[#bc5d5d]■[#bd5f5f]■[#be6262]■[#bf6464]▪[#e2e5ea]■[/] [b][#e8eaed]81%[/] [b][#e8eaed]2d07h[/]",
                " [b][#e8eaed]|[/] [u][#e8eaed]Opus 4[/][b][#e8eaed]:[/][b][#8290cc]h[b][#8b8fcb]i[b][#938dcb]g[b][#9c8cca]h[/]\n",
                "[b][#e8eaed]ram[/] [#be823c]■[#c08541]■[#c28947]■[#c38c4c]■[#c59051]■[#c79357]▪[#e2e5ea]■[#e2e5ea]■[/] [b][#e8eaed]12.0/16.0G[/]",
                " [b][#e8eaed]|[/] [b][#e8eaed]{disk}[/] [#c83c3c]■[#c94040]■[#cb4545]■[#cc4949]■[#cd4e4e]■[#ce5252]■[#d05757]▪[#e2e5ea]■[/] [b][#e8eaed]90%[/]",
                " [b][#e8eaed]|[/] [b][#e8eaed]cpu[/] [#375a8c]▪[#e2e5ea]■[#e2e5ea]■[#e2e5ea]■[#e2e5ea]■[#e2e5ea]■[#e2e5ea]■[#e2e5ea]■[/] [b][#e8eaed]12%[/]",
                " [b][#e8eaed]|[/] [b][#e8eaed]update[/][/]\n",
            ),
            disk = DISK
        );
        assert_eq!(line(false, None, &sample()), expected);
    }

    #[test]
    fn light_line() {
        let expected = format!(
            concat!(
                "[dim][#000000]ª[/][b][#000000]ctx[/] [#7db9ff]■[#71abf4]■[#659eea]▪[#beb4a5]■[#beb4a5]■[#beb4a5]■[#beb4a5]■[#beb4a5]■[/] [b][#000000]60k/200k[/]",
                " [b][#000000]|[/] [b][#000000]5h[/] [#aa5aaf]■[#9a519e]■[#8a498d]▪[#beb4a5]■[#beb4a5]■[#beb4a5]■[#beb4a5]■[#beb4a5]■[/] [b][#000000]36%[/] [b][#000000]1h00m[/]",
                " [b][#000000]|[/] [b][#000000]7d[/] [#b95555]■[#b65454]■[#b35252]■[#af5151]■[#ac4f4f]■[#a94e4e]■[#a64c4c]▪[#beb4a5]■[/] [b][#000000]81%[/] [b][#000000]2d07h[/]",
                " [b][#000000]|[/] [u][#000000]Opus 4[/][b][#000000]:[/][b][#4b5476]h[b][#505376]i[b][#555275]g[b][#5a5175]h[/]\n",
                "[b][#000000]ram[/] [#be823c]■[#b87e3a]■[#b27a38]■[#ac7636]■[#a67234]■[#a06e33]▪[#beb4a5]■[#beb4a5]■[/] [b][#000000]12.0/16.0G[/]",
                " [b][#000000]|[/] [b][#000000]{disk}[/] [#c83c3c]■[#c33a3a]■[#be3939]■[#b83737]■[#b33636]■[#ae3434]■[#a93333]▪[#beb4a5]■[/] [b][#000000]90%[/]",
                " [b][#000000]|[/] [b][#000000]cpu[/] [#375a8c]▪[#beb4a5]■[#beb4a5]■[#beb4a5]■[#beb4a5]■[#beb4a5]■[#beb4a5]■[#beb4a5]■[/] [b][#000000]12%[/]",
                " [b][#000000]|[/] [b][#000000]update[/][/]\n",
            ),
            disk = DISK
        );
        assert_eq!(line(true, None, &sample()), expected);
    }

    #[test]
    fn quota_bar_levels() {
        let levels = [Some(0.0), Some(50.0), Some(80.0), Some(100.0), None].map(quota_only);
        assert_eq!(
            levels,
            [
                "[b][#e8eaed]5h[/] [#e2e5ea]■[#e2e5ea]■[#e2e5ea]■[#e2e5ea]■[#e2e5ea]■[#e2e5ea]■[#e2e5ea]■[#e2e5ea]■[/] [b][#e8eaed]0%[/] [b][#e8eaed]--[/][/]\n",
                "[b][#e8eaed]5h[/] [#aa5aaf]■[#af63b3]■[#b46cb8]■[#b876bc]▪[#e2e5ea]■[#e2e5ea]■[#e2e5ea]■[#e2e5ea]■[/] [b][#e8eaed]50%[/] [b][#e8eaed]--[/][/]\n",
                "[b][#e8eaed]5h[/] [#c83c5f]■[#ca4163]■[#cb4768]■[#cd4c6c]■[#ce5271]■[#d05775]■[#d15c79]▪[#e2e5ea]■[/] [b][#e8eaed]80%[/] [b][#e8eaed]--[/][/]\n",
                "[b][#e8eaed]5h[/] [#c83c5f]■[#c94163]■[#cb4567]■[#cc4a6a]■[#cd4f6e]■[#cf5372]■[#d05876]■[#d15c79]▪[/] [b][#d15c79]100%[/] [b][#e8eaed]--[/][/]\n",
                "[b][#e8eaed]5h[/] [#e2e5ea]■■■■■■■■[/] [b][#e8eaed]--%[/] [b][#e8eaed]--[/][/]\n",
            ]
        );
    }
}
