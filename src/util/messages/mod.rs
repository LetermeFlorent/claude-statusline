mod check;
mod help;
mod language;
mod table;
#[cfg(test)]
mod tests;

pub use check::*;
pub use help::HELP;
pub use table::*;

use alloc::string::String;
#[cfg(any(windows, test))]
use core::fmt;
use language::Language;

const PLACEHOLDER: &str = "{}";

pub fn language_code() -> &'static str {
    language::current().code()
}

#[cfg(test)]
pub fn in_french<R>(run: impl FnOnce() -> R) -> R {
    language::with_test_language(Language::French, run)
}

pub struct Text {
    english: &'static str,
    french: &'static str,
}

impl Text {
    pub const fn new(english: &'static str, french: &'static str) -> Text {
        Text { english, french }
    }

    pub fn text(&self) -> &'static str {
        self.in_language(language::current())
    }

    pub fn with(&self, values: &[&str]) -> String {
        fill(self.text(), values)
    }

    #[cfg(any(windows, test))]
    pub fn write_to(&self, out: &mut dyn fmt::Write, value: &dyn fmt::Display) -> fmt::Result {
        match self.text().split_once(PLACEHOLDER) {
            Some((before, after)) => write!(out, "{}{}{}", before, value, after),
            None => out.write_str(self.text()),
        }
    }

    fn in_language(&self, language: Language) -> &'static str {
        match language {
            Language::English => self.english,
            Language::French => self.french,
        }
    }
}

fn fill(template: &str, values: &[&str]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut values = values.iter();
    let mut rest = template;
    while let Some(at) = rest.find(PLACEHOLDER) {
        out.push_str(&rest[..at]);
        out.push_str(values.next().copied().unwrap_or_default());
        rest = &rest[at + PLACEHOLDER.len()..];
    }
    out.push_str(rest);
    out
}
