use crate::config::{Config, Metric};
use crate::render::line::GIB;
use crate::render::{render, Ctx, Data, Depth, Quota, Quotas, SysValue, Terminal};
use alloc::string::String;
use core::fmt::Write;

pub(super) const NOW: f64 = 1_000_000.0;

pub(super) const DISK: &str = if cfg!(windows) { "C:" } else { "dsk" };

pub(super) fn readable(rendered: &str) -> String {
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

pub(super) fn sample() -> Data {
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
        spend: None,
        cache_warm: None,
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

pub(super) fn line(light: bool, config: Option<&str>, data: &Data) -> String {
    let cfg = Config::parse(config);
    readable(&render(
        &cfg,
        &cfg.colors(light),
        data,
        NOW,
        &Terminal::WIDE,
    ))
}

pub(super) fn fitted(columns: usize, config: Option<&str>) -> String {
    let cfg = Config::parse(config);
    let term = Terminal {
        columns: Some(columns),
        depth: Depth::TrueColor,
    };
    readable(&render(&cfg, &cfg.colors(false), &sample(), NOW, &term))
}

pub(super) fn raw(depth: Depth) -> String {
    let cfg = Config::parse(None);
    let term = Terminal {
        columns: None,
        depth,
    };
    render(&cfg, &cfg.colors(false), &sample(), NOW, &term)
}
