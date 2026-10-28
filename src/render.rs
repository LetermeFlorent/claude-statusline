use crate::config::{Colors, Config, Metric, Palette, Rgb};
use crate::num;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Quota {
    pub pct: Option<f64>,
    pub at: Option<f64>,
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
    pub quotas: Option<(Option<Quota>, Option<Quota>)>,
    pub model: String,
    pub effort: Option<String>,
    pub sys: Vec<SysValue>,
    pub updating: bool,
}

const RESET: &str = "\x1b[0m";

type Pair = (Rgb, Rgb);

fn effort_pair(light: bool, key: &str) -> Pair {
    let l: [(&str, Pair); 6] = [
        ("low", ([70.0, 100.0, 97.0], [72.0, 92.0, 114.0])),
        ("medium", ([71.0, 90.0, 115.0], [77.0, 85.0, 117.0])),
        ("high", ([75.0, 84.0, 118.0], [90.0, 81.0, 117.0])),
        ("xhigh", ([88.0, 80.0, 118.0], [102.0, 78.0, 116.0])),
        ("max", ([102.0, 78.0, 116.0], [114.0, 80.0, 114.0])),
        ("", ([116.0, 80.0, 117.0], [128.0, 93.0, 121.0])),
    ];
    let d: [(&str, Pair); 6] = [
        ("low", ([120.0, 172.0, 168.0], [124.0, 158.0, 196.0])),
        ("medium", ([122.0, 156.0, 198.0], [132.0, 146.0, 202.0])),
        ("high", ([130.0, 144.0, 204.0], [156.0, 140.0, 202.0])),
        ("xhigh", ([152.0, 138.0, 204.0], [176.0, 134.0, 200.0])),
        ("max", ([176.0, 134.0, 200.0], [196.0, 138.0, 196.0])),
        ("", ([200.0, 138.0, 202.0], [220.0, 160.0, 208.0])),
    ];
    let t = if light { &l } else { &d };
    t.iter().find(|(k, _)| *k == key).unwrap_or(&t[5]).1
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
    s: String,
    cfg: &'a Config,
    col: &'a Colors,
}

impl Out<'_> {
    fn fg(&mut self, c: Rgb) {
        let _ = write!(
            self.s,
            "\x1b[38;2;{};{};{}m",
            c[0] as i64, c[1] as i64, c[2] as i64
        );
    }

    fn bold(&mut self) {
        if self.cfg.bold {
            self.s.push_str("\x1b[1m");
        }
    }

    fn strong(&mut self, text: &str, c: Rgb) {
        self.bold();
        self.fg(c);
        self.s.push_str(text);
        self.s.push_str(RESET);
    }

    fn label(&mut self, text: &str) {
        let ink = self.col.ink;
        self.strong(text, ink);
    }

    // Une cellule par couleur ; une barre sans valeur est un seul bloc vide
    fn bar(&mut self, p: Option<f64>, color_at: &dyn Fn(usize, usize) -> Rgb) -> Option<Rgb> {
        let w = self.cfg.width;
        let empty = self.col.empty;
        let Some(p) = p else {
            self.fg(empty);
            for _ in 0..w {
                self.s.push_str(&self.cfg.empty);
            }
            self.s.push_str(RESET);
            return None;
        };
        let n = cell_count(p, w);
        let mut last = None;
        for i in 0..w {
            if i < n {
                let c = color_at(i, n);
                last = Some(c);
                self.fg(c);
                let g = if i == n - 1 {
                    &self.cfg.half
                } else {
                    &self.cfg.filled
                };
                self.s.push_str(g);
            } else {
                self.fg(empty);
                self.s.push_str(&self.cfg.empty);
            }
        }
        self.s.push_str(RESET);
        last
    }

    fn gradient_bar(&mut self, p: Option<f64>, start: Rgb, g: f64) -> Option<Rgb> {
        let light = self.col.light;
        let target = start.map(|c| {
            num::round(if light {
                c * 0.65
            } else {
                c + (255.0 - c) * 0.3
            })
        });
        let end = lerp(start, target, g);
        self.bar(p, &|i, n| {
            rgb(lerp(
                start,
                end,
                if n > 1 {
                    i as f64 / (n - 1) as f64
                } else {
                    0.0
                },
            ))
        })
    }

    fn value(&mut self, text: &str, p: Option<f64>, last: Option<Rgb>) {
        let c = match (p, last) {
            (Some(p), Some(l)) if num::round(p) >= 100.0 => l,
            _ => self.col.ink,
        };
        self.strong(text, c);
    }

    fn ctx(&mut self, d: &Data) {
        let size = d.ctx.size.filter(|s| *s > 0.0);
        let mut pct = d.ctx.pct;
        let used: Option<f64> = match (d.ctx.tokens, pct, size) {
            (Some(t), _, _) => Some(t as f64),
            (None, Some(p), Some(s)) => Some((p * s / 100.0) as u64 as f64),
            _ => None,
        };
        let mut denom = size;
        let mut marked = false;
        if let (Some(s), Some(w)) = (size, d.window) {
            if w > 0 && (w as f64) < s {
                denom = Some(w as f64);
                pct = used.map(|u| u / w as f64 * 100.0);
                marked = true;
            }
        }
        if marked {
            self.s.push_str("\x1b[2m");
            let ink = self.col.ink;
            self.fg(ink);
            self.s.push_str(&self.cfg.marker);
            self.s.push_str(RESET);
        }
        self.label("ctx");
        self.s.push(' ');
        let (from, to) = (self.col.ctx_from, self.col.ctx_to);
        let span = if self.cfg.width > 1 {
            (self.cfg.width - 1) as f64
        } else {
            1.0
        };
        let last = self.bar(pct, &|i, _| rgb(lerp(from, to, i as f64 / span)));
        self.s.push(' ');
        let text = match (denom, used) {
            (Some(dn), Some(u)) => format!("{}/{}", tokens(u as u64), tokens(dn as u64)),
            _ => pct_text(pct),
        };
        self.value(&text, pct, last);
    }

    fn quota(&mut self, name: &str, q: Option<Quota>, pal: Palette, g: f64, now: f64) {
        let q = q.unwrap_or_default();
        self.label(name);
        self.s.push(' ');
        let start = match q.pct {
            Some(p) if p >= self.cfg.quota_hot => pal.hot,
            Some(p) if p >= self.cfg.quota_warn => pal.warn,
            _ => pal.ok,
        };
        let last = self.gradient_bar(q.pct, start, g);
        self.s.push(' ');
        self.value(&pct_text(q.pct), q.pct, last);
        if let Some(at) = q.at {
            self.s.push(' ');
            self.label(&reset_text(at - now, name == "7d"));
        } else if name == "5h" {
            self.s.push(' ');
            self.label("--");
        }
    }

    fn model(&mut self, d: &Data) {
        self.s.push_str("\x1b[4m");
        let ink = self.col.ink;
        self.fg(ink);
        self.s.push_str(&d.model);
        self.s.push_str(RESET);
        let Some(level) = d.effort.as_deref() else {
            return;
        };
        let mut key = level.to_lowercase();
        if key == "auto" {
            key = String::from("low");
        }
        let word = if key == "ultracode" {
            format!("{}(+workflows)", level)
        } else {
            String::from(level)
        };
        let (start, end) = effort_pair(self.col.light, &key);
        self.label(":");
        if self.cfg.effort_flat {
            self.strong(&word, end);
            return;
        }
        let chars: Vec<char> = word.chars().collect();
        for (i, ch) in chars.iter().enumerate() {
            let t = if chars.len() > 1 {
                i as f64 / (chars.len() - 1) as f64
            } else {
                0.0
            };
            self.bold();
            self.fg(rgb(lerp(start, end, t)));
            self.s.push(*ch);
        }
        self.s.push_str(RESET);
    }

    fn system(&mut self, v: &SysValue) {
        let cfg = self.cfg;
        let (name, pal, p, text) = match v.metric {
            Metric::Ram => {
                let p = v
                    .ram
                    .filter(|(t, _)| *t > 0.0)
                    .map(|(t, a)| (t - a) / t * 100.0);
                let text = match (p, v.ram) {
                    (Some(_), Some((t, a))) => {
                        let gib = 1073741824.0;
                        format!("{:.1}/{:.1}G", (t - a) / gib, t / gib)
                    }
                    _ => String::from("--%"),
                };
                (String::from("ram"), cfg.pal_ram, p, text)
            }
            Metric::Io => (cfg.disk_label(), cfg.pal_dsk, v.pct, pct_text(v.pct)),
            Metric::Cpu => (String::from("cpu"), cfg.pal_cpu, v.pct, pct_text(v.pct)),
        };
        self.label(&name);
        self.s.push(' ');
        let start = match p {
            Some(p) if p > cfg.sys_hot => pal.hot,
            Some(p) if p > cfg.sys_warn => pal.warn,
            _ => pal.ok,
        };
        let last = self.gradient_bar(p, start, cfg.grad_sys);
        self.s.push(' ');
        self.value(&text, p, last);
    }

    fn sep(&mut self, first: &mut bool) {
        if !*first {
            self.s.push(' ');
            self.label("|");
            self.s.push(' ');
        }
        *first = false;
    }
}

pub fn cell_count(p: f64, w: usize) -> usize {
    if !(p > 0.0) {
        return 0;
    }
    let units = num::round(p * 2.0 * w as f64 / 100.0);
    let n = num::ceil(units / 2.0);
    let n = if n < 1.0 { 1.0 } else { n };
    if n > w as f64 {
        w
    } else {
        n as usize
    }
}

pub fn pct_text(p: Option<f64>) -> String {
    match p {
        // Une valeur negative s'affiche 0%
        Some(p) => format!("{}%", num::round(p) as u64),
        None => String::from("--%"),
    }
}

pub fn tokens(n: u64) -> String {
    let f = n as f64;
    if n < 1000 {
        format!("{}", n)
    } else if f < 1e6 {
        format!("{:.0}k", f / 1e3)
    } else if f < 1e9 {
        format!("{:.0}M", f / 1e6)
    } else {
        format!("{:.0}B", f / 1e9)
    }
}

pub fn reset_text(secs: f64, long: bool) -> String {
    let s = num::floor(secs) as i64;
    if s <= 0 {
        return String::from("0m");
    }
    if s < 3600 {
        return format!("{}m", s / 60);
    }
    if long && s >= 86400 {
        return format!("{}d{:02}h", s / 86400, (s % 86400) / 3600);
    }
    format!("{}h{:02}m", s / 3600, (s % 3600) / 60)
}

pub fn render(cfg: &Config, col: &Colors, d: &Data, now: f64) -> String {
    let mut o = Out {
        s: String::with_capacity(2048),
        cfg,
        col,
    };
    let mut first = true;
    if cfg.seg_ctx {
        o.sep(&mut first);
        o.ctx(d);
    }
    if let Some((q5, q7)) = d.quotas {
        if cfg.seg_5h {
            o.sep(&mut first);
            o.quota("5h", q5, cfg.pal_5h, cfg.grad_5h, now);
        }
        if cfg.seg_7d {
            o.sep(&mut first);
            o.quota("7d", q7, cfg.pal_7d, cfg.grad_7d, now);
        }
    }
    if cfg.seg_model {
        o.sep(&mut first);
        o.model(d);
    }
    if !d.sys.is_empty() || d.updating {
        o.s.push('\n');
        first = true;
        for v in &d.sys {
            o.sep(&mut first);
            o.system(v);
        }
        if d.updating {
            o.sep(&mut first);
            o.label("update");
        }
    }
    o.s.push_str(RESET);
    o.s.push('\n');
    o.s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn textes() {
        assert_eq!(tokens(2500), "2k");
        assert_eq!(tokens(3500), "4k");
        assert_eq!(tokens(999), "999");
        assert_eq!(tokens(u64::MAX), "18446744074B");
        assert_eq!(reset_text(59.9, false), "0m");
        assert_eq!(reset_text(5000.0, false), "1h23m");
        assert_eq!(reset_text(200000.0, true), "2d07h");
        assert_eq!(reset_text(-5.0, true), "0m");
        assert_eq!(cell_count(50.0, 8), 4);
        assert_eq!(cell_count(0.5, 8), 1);
        assert_eq!(cell_count(150.0, 8), 8);
    }

    const NOW: f64 = 1_000_000.0;
    const GIB: f64 = 1073741824.0;

    fn readable(s: &str) -> String {
        let mut parts = s.split('\x1b');
        let mut out = String::from(parts.next().unwrap_or(""));
        for part in parts {
            let (code, text) = part[1..].split_once('m').unwrap();
            match code {
                "0" => out.push_str("[/]"),
                "1" => out.push_str("[b]"),
                "2" => out.push_str("[dim]"),
                "4" => out.push_str("[u]"),
                _ => {
                    let c: Vec<u8> = code
                        .strip_prefix("38;2;")
                        .unwrap()
                        .split(';')
                        .map(|x| x.parse().unwrap())
                        .collect();
                    let _ = write!(out, "[#{:02x}{:02x}{:02x}]", c[0], c[1], c[2]);
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
            quotas: Some((
                Some(Quota {
                    pct: Some(36.0),
                    at: Some(NOW + 3600.0),
                }),
                Some(Quota {
                    pct: Some(81.0),
                    at: Some(NOW + 200_000.0),
                }),
            )),
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

    fn line(light: bool, config: Option<&str>, d: &Data) -> String {
        let cfg = Config::parse(config);
        readable(&render(&cfg, &cfg.colors(light), d, NOW))
    }

    fn quota_only(pct: Option<f64>) -> String {
        let mut d = sample();
        d.quotas = Some((Some(Quota { pct, at: None }), None));
        d.sys.clear();
        d.updating = false;
        let only_5h = "{\"segments\": {\"ctx\": false, \"7d\": false, \"model\": false}}";
        line(false, Some(only_5h), &d)
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
