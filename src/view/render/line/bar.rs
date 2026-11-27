use super::{Out, RESET};
use crate::config::Rgb;
use crate::num;
use crate::render::screen::fill;

impl Out<'_> {
    pub(super) fn bar(&mut self, pct: Option<f64>, color: Rgb) -> Option<Rgb> {
        let width = self.layout.width;
        let filled = pct.map_or(0, |p| fill::cells(p, width));
        if filled > 0 {
            self.fg(color);
            for _ in 0..filled {
                self.line.push_str(&self.cfg.filled);
            }
        }
        if filled < width {
            let empty = self.col.empty;
            self.fg(empty);
            for _ in filled..width {
                self.line.push_str(&self.cfg.empty);
            }
        }
        self.line.push_str(RESET);
        (filled > 0).then_some(color)
    }

    pub(super) fn value(&mut self, text: &str, pct: Option<f64>, last: Option<Rgb>) {
        let color = match (pct, last) {
            (Some(p), Some(l)) if num::round(p) >= 100.0 => Some(l),
            _ => self.col.ink,
        };
        self.strong(text, color);
    }

    pub(super) fn percent(
        &mut self,
        text: &str,
        field: usize,
        pct: Option<f64>,
        last: Option<Rgb>,
    ) {
        for _ in text.chars().count()..field {
            self.line.push(' ');
        }
        self.value(text, pct, last);
    }
}
