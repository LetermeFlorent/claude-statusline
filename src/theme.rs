pub use detect::{detect, detect_light};
pub use hosts::color::colorfgbg;
#[cfg(not(windows))]
pub use hosts::color::luminance_light;
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
