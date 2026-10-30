pub use detect::{detect, detect_light};
pub use hosts::color::colorfgbg;
pub use source::{Decision, Source};

mod cache;
mod detect;
mod hosts;
mod locate;
mod lookup;
mod names;
mod source;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Light,
    Dark,
    System,
}

type Lookup<'a> = dyn Fn(&str) -> Option<Mode> + 'a;

pub fn luminance_light(c: [f64; 3]) -> bool {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2] > 128.0
}
