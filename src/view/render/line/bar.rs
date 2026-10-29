use super::{lerp, rgb, Out, RESET};
use crate::config::Rgb;
use crate::num;
use crate::render::screen::fill;

const MAX_CHANNEL: f64 = 255.0;
const LIGHT_GRADIENT_SHADE: f64 = 0.65;
const DARK_GRADIENT_TINT: f64 = 0.3;

impl Out<'_> {
    pub(super) fn bar(
        &mut self,
        pct: Option<f64>,
        color_at: &dyn Fn(usize, usize) -> Rgb,
    ) -> Option<Rgb> {
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

    pub(super) fn gradient_bar(
        &mut self,
        pct: Option<f64>,
        start: Rgb,
        gradient: f64,
    ) -> Option<Rgb> {
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

    pub(super) fn value(&mut self, text: &str, pct: Option<f64>, last: Option<Rgb>) {
        let color = match (pct, last) {
            (Some(p), Some(l)) if num::round(p) >= 100.0 => Some(l),
            _ => self.col.ink,
        };
        self.strong(text, color);
    }
}
