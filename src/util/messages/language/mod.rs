#[cfg(not(windows))]
mod apple;
#[cfg(any(windows, test))]
mod identifier;
#[cfg(any(not(windows), test))]
mod locale;
#[cfg(test)]
mod tests;

use alloc::string::String;
#[cfg(test)]
use core::cell::Cell;
use core::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Language {
    English,
    French,
}

pub const CODE_ENDS: [char; 4] = ['_', '.', '@', '-'];
const FORCED: &str = "STATUSLINE_LANG";
const UNDECIDED: u8 = 0;
const ENGLISH: u8 = 1;
const FRENCH: u8 = 2;

static CHOSEN: AtomicU8 = AtomicU8::new(UNDECIDED);

#[cfg(test)]
std::thread_local! {
    static TEST_LANGUAGE: Cell<Language> = const { Cell::new(Language::English) };
}

impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::French => "fr",
        }
    }
}

pub fn from_tag(tag: &str) -> Option<Language> {
    let code = tag.trim().split(CODE_ENDS).next()?;
    if code.eq_ignore_ascii_case("fr") {
        Some(Language::French)
    } else if code.eq_ignore_ascii_case("en") {
        Some(Language::English)
    } else {
        None
    }
}

pub fn choose(
    variable: &dyn Fn(&str) -> Option<String>,
    system: &dyn Fn() -> Option<Language>,
) -> Language {
    variable(FORCED)
        .and_then(|value| from_tag(&value))
        .or_else(system)
        .unwrap_or(Language::English)
}

#[cfg(windows)]
fn system() -> Option<Language> {
    Some(identifier::from_language_id(crate::os::ui_language()))
}

#[cfg(not(windows))]
fn system() -> Option<Language> {
    locale::from_locale(&crate::os::env).or_else(apple::preferred)
}

#[cfg(test)]
fn test_language() -> Option<Language> {
    Some(TEST_LANGUAGE.get())
}

#[cfg(not(test))]
fn test_language() -> Option<Language> {
    None
}

#[cfg(test)]
pub fn with_test_language<R>(language: Language, run: impl FnOnce() -> R) -> R {
    let before = TEST_LANGUAGE.replace(language);
    let result = run();
    TEST_LANGUAGE.set(before);
    result
}

pub fn current() -> Language {
    if let Some(language) = test_language() {
        return language;
    }
    match CHOSEN.load(Ordering::Relaxed) {
        ENGLISH => Language::English,
        FRENCH => Language::French,
        _ => {
            let chosen = choose(&crate::os::env, &system);
            let mark = if chosen == Language::French {
                FRENCH
            } else {
                ENGLISH
            };
            CHOSEN.store(mark, Ordering::Relaxed);
            chosen
        }
    }
}
