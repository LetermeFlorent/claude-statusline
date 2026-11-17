use super::tone::{Pair, Tone};
use super::{lerp, rgb, Out, RESET};
use crate::render::Data;
use alloc::format;
use alloc::string::String;

impl Out<'_> {
    pub(super) fn model(&mut self, data: &Data) {
        let Some(level) = data.effort.as_deref() else {
            self.ink(&data.model);
            return;
        };
        let key = level.to_lowercase();
        let pair = Tone::of(&key).pair(self.col.light);
        if !self.layout.effort {
            self.shade(&data.model, pair, false);
            return;
        }
        self.ink(&data.model);
        let word = if key == "ultracode" {
            format!("{}(+workflows)", level)
        } else {
            String::from(level)
        };
        self.label(":");
        self.shade(&word, pair, true);
    }

    fn ink(&mut self, text: &str) {
        let ink = self.col.ink;
        self.tint(ink);
        self.line.push_str(text);
        self.line.push_str(RESET);
    }

    fn shade(&mut self, text: &str, (start, end): Pair, bold: bool) {
        let count = text.chars().count();
        for (i, letter) in text.chars().enumerate() {
            let progress = match (self.cfg.effort_flat, count > 1) {
                (true, _) => 1.0,
                (false, true) => i as f64 / (count - 1) as f64,
                (false, false) => 0.0,
            };
            if bold {
                self.bold();
            }
            self.fg(rgb(lerp(start, end, progress)));
            self.line.push(letter);
        }
        self.line.push_str(RESET);
    }
}
