use super::Out;
use crate::config::Metric;
use crate::messages;
use crate::render::text::{decimal, pct_text, whole, PERCENT_SCALE};
use crate::render::SysValue;
use alloc::string::String;

const SYS_FIELD: usize = 4;

pub(in crate::view::render) const GIB: f64 = 1_073_741_824.0;

impl Out<'_> {
    pub(super) fn system(&mut self, reading: &SysValue) {
        let cfg = self.cfg;
        let (name, palette, pct, text, field) = match reading.metric {
            Metric::Ram => {
                let pct = reading
                    .ram
                    .filter(|(total, _)| *total > 0.0)
                    .map(|(total, available)| (total - available) / total * PERCENT_SCALE);
                let text = match (pct, reading.ram) {
                    (Some(_), Some((total, available))) => messages::RAM_USAGE
                        .with(&[&decimal((total - available) / GIB), &decimal(total / GIB)]),
                    _ => pct_text(None),
                };
                (String::from("ram"), cfg.pal_ram, pct, text, 0)
            }
            Metric::Io => (
                cfg.disk_label(),
                cfg.pal_dsk,
                reading.pct,
                pct_text(reading.pct),
                SYS_FIELD,
            ),
            Metric::Cpu => (
                String::from("cpu"),
                cfg.pal_cpu,
                reading.pct,
                pct_text(reading.pct),
                SYS_FIELD,
            ),
        };
        self.label(&name);
        self.line.push(' ');
        let color = match pct {
            Some(p) if whole(p) > cfg.sys_hot => palette.hot,
            Some(p) if whole(p) > cfg.sys_warn => palette.warn,
            _ => palette.ok,
        };
        let last = self.bar(pct, color);
        self.line.push(' ');
        self.percent(&text, field, pct, last);
    }
}
