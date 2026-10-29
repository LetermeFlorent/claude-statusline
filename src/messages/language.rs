use alloc::string::String;
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

#[cfg(any(windows, test))]
pub fn from_language_id(id: u16) -> Language {
    const PRIMARY_LANGUAGE_MASK: u16 = 0x3ff;
    const LANG_FRENCH: u16 = 0x0c;
    if id & PRIMARY_LANGUAGE_MASK == LANG_FRENCH {
        Language::French
    } else {
        Language::English
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
    Some(from_language_id(crate::os::ui_language()))
}

#[cfg(not(windows))]
fn system() -> Option<Language> {
    super::locale::from_locale(&crate::os::env).or_else(super::apple::preferred)
}

pub fn current() -> Language {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn forced(value: &str) -> Language {
        let variable = |name: &str| (name == FORCED).then(|| String::from(value));
        choose(&variable, &|| Some(Language::English))
    }

    #[test]
    fn french_windows_display_languages_pick_french() {
        assert_eq!(from_language_id(0x040c), Language::French);
        assert_eq!(from_language_id(0x0c0c), Language::French);
        assert_eq!(from_language_id(0x100c), Language::French);
    }

    #[test]
    fn other_windows_display_languages_pick_english() {
        assert_eq!(from_language_id(0x0409), Language::English);
        assert_eq!(from_language_id(0x0407), Language::English);
        assert_eq!(from_language_id(0x0462), Language::English);
        assert_eq!(from_language_id(0), Language::English);
    }

    #[test]
    fn statusline_lang_comes_before_the_system() {
        let variable = |name: &str| (name == FORCED).then(|| String::from("fr"));
        assert_eq!(
            choose(&variable, &|| Some(Language::English)),
            Language::French
        );
        let variable = |name: &str| (name == FORCED).then(|| String::from("en"));
        assert_eq!(
            choose(&variable, &|| Some(Language::French)),
            Language::English
        );
    }

    #[test]
    fn statusline_lang_ignores_case_and_a_region_suffix() {
        for value in ["FR", " fr ", "fr_FR.UTF-8", "fr-CA"] {
            assert_eq!(forced(value), Language::French, "{}", value);
        }
    }

    #[test]
    fn an_unknown_statusline_lang_falls_back_to_the_system() {
        let variable = |name: &str| (name == FORCED).then(|| String::from("de"));
        assert_eq!(
            choose(&variable, &|| Some(Language::French)),
            Language::French
        );
        let variable = |name: &str| (name == FORCED).then(String::new);
        assert_eq!(
            choose(&variable, &|| Some(Language::French)),
            Language::French
        );
    }

    #[test]
    fn nothing_known_means_english() {
        assert_eq!(choose(&|_| None, &|| None), Language::English);
    }

    #[test]
    fn language_tags_with_a_dash_are_read_by_their_first_part() {
        assert_eq!(from_tag("fr-FR"), Some(Language::French));
        assert_eq!(from_tag("fr-Latn-CA"), Some(Language::French));
        assert_eq!(from_tag("en-US"), Some(Language::English));
        assert_eq!(from_tag("de-DE"), None);
        assert_eq!(from_tag(""), None);
    }
}
