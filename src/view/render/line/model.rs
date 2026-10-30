use super::tone::Tone;
use super::{lerp, rgb, Out, RESET};
use crate::render::Data;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

impl Out<'_> {
    pub(super) fn model(&mut self, data: &Data) {
        self.line.push_str("\x1b[4m");
        let ink = self.col.ink;
        self.tint(ink);
        self.line.push_str(&data.model);
        self.line.push_str(RESET);
        let Some(level) = data.effort.as_deref().filter(|_| self.layout.effort) else {
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
            self.strong(&word, Some(end));
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
}
