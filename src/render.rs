mod data;
pub mod demo;
mod line;
mod screen;
#[cfg(test)]
mod tests;
mod text;

use crate::config::{Colors, Config};
use alloc::string::String;
use fit::Layout;
use screen::fit;

pub use data::{Ctx, Data, Quota, Quotas, SysValue};
pub use screen::{Depth, Terminal};
#[cfg(test)]
pub use text::MAX_LABEL_CHARS;
pub use text::{label_text, printable};

pub fn render(cfg: &Config, col: &Colors, data: &Data, now: f64, term: &Terminal) -> String {
    let mut layout = Layout::full(cfg.width);
    loop {
        let text = line::draw(cfg, col, data, now, layout, term.depth);
        let fits = term
            .columns
            .is_none_or(|columns| fit::widest(&text) <= columns);
        match layout.narrower() {
            Some(narrower) if !fits => layout = narrower,
            _ => return text,
        }
    }
}
