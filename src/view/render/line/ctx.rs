use super::Out;
use crate::config::{Rgb, HOT_SYS, WARN_SYS};
use crate::messages;
use crate::render::text::{pct_text, tokens, whole, PERCENT_SCALE};
use crate::render::Data;
use alloc::format;

impl Out<'_> {
    pub(super) fn ctx(&mut self, data: &Data) {
        let size = data.ctx.size.filter(|s| *s > 0.0);
        let mut pct = data.ctx.pct;
        let used: Option<f64> = match (data.ctx.tokens, pct, size) {
            (Some(t), _, _) => Some(t as f64),
            (None, Some(p), Some(s)) => Some((p * s / PERCENT_SCALE) as u64 as f64),
            _ => None,
        };
        let mut denom = size;
        let mut marked = false;
        if let (Some(size), Some(window)) = (size, data.window) {
            if window > 0 && (window as f64) < size {
                denom = Some(window as f64);
                pct = used.map(|u| u / window as f64 * PERCENT_SCALE);
                marked = true;
            }
        }
        if marked && !self.cfg.marker.is_empty() {
            let marker = self.cfg.marker.clone();
            self.faint(&marker);
        }
        self.label("ctx");
        self.line.push(' ');
        let last = self.ctx_bar(pct);
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

    fn ctx_bar(&mut self, pct: Option<f64>) -> Option<Rgb> {
        let color = match pct {
            Some(p) if whole(p) >= self.cfg.ctx_hot => HOT_SYS,
            Some(p) if whole(p) >= self.cfg.ctx_warn => WARN_SYS,
            _ => self.col.ctx_from,
        };
        self.bar(pct, color)
    }
}
