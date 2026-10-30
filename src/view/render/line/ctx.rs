use super::{lerp, rgb, Out};
use crate::config::{HOT_SYS, WARN_SYS};
use crate::messages;
use crate::render::text::{pct_text, tokens};
use crate::render::Data;
use alloc::format;

impl Out<'_> {
    pub(super) fn ctx(&mut self, data: &Data) {
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
            self.faint(&marker);
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
}
