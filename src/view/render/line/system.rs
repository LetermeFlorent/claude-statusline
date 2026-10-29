use super::Out;
use crate::config::Metric;
use crate::messages;
use crate::render::text::{decimal, pct_text};
use crate::render::SysValue;
use alloc::string::String;

pub(in crate::view::render) const GIB: f64 = 1_073_741_824.0;

impl Out<'_> {
    pub(super) fn system(&mut self, reading: &SysValue) {
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
}
