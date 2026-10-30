mod bar;
mod ctx;
mod draw;
mod model;
mod quota;
mod system;
mod tone;

pub(super) use draw::draw;
#[cfg(test)]
pub(in crate::view::render) use system::GIB;

use crate::config::{Colors, Config, Rgb};
use crate::num;
use crate::render::screen::fit::Layout;
use crate::render::screen::palette;
use crate::render::Depth;
use alloc::string::String;
use core::fmt::Write;

const RESET: &str = "\x1b[0m";

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
    line: String,
    cfg: &'a Config,
    col: &'a Colors,
    layout: Layout,
    depth: Depth,
}

impl Out<'_> {
    fn fg(&mut self, c: Rgb) {
        let _ = match self.depth {
            Depth::TrueColor => write!(
                self.line,
                "\x1b[38;2;{};{};{}m",
                c[0] as i64, c[1] as i64, c[2] as i64
            ),
            Depth::Indexed => write!(self.line, "\x1b[38;5;{}m", palette::indexed(c)),
        };
    }

    fn bold(&mut self) {
        if self.cfg.bold {
            self.line.push_str("\x1b[1m");
        }
    }

    fn tint(&mut self, c: Option<Rgb>) {
        if let Some(c) = c {
            self.fg(c);
        }
    }

    fn strong(&mut self, text: &str, c: Option<Rgb>) {
        self.bold();
        self.tint(c);
        self.line.push_str(text);
        self.line.push_str(RESET);
    }

    fn label(&mut self, text: &str) {
        let ink = self.col.ink;
        self.strong(text, ink);
    }

    fn faint(&mut self, text: &str) {
        self.line.push_str("\x1b[2m");
        let ink = self.col.ink;
        self.tint(ink);
        self.line.push_str(text);
        self.line.push_str(RESET);
    }

    fn sep(&mut self, first: &mut bool) {
        if !*first {
            self.line.push(' ');
            self.label("|");
            self.line.push(' ');
        }
        *first = false;
    }
}
