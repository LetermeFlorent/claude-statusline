#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Language {
    English,
    French,
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

#[cfg(any(not(windows), test))]
pub fn from_locale(variable: &dyn Fn(&str) -> Option<alloc::string::String>) -> Language {
    const VARIABLES: [&str; 3] = ["LC_ALL", "LC_MESSAGES", "LANG"];
    const CODE_ENDS: [char; 3] = ['_', '.', '@'];
    let locale = VARIABLES
        .iter()
        .find_map(|name| variable(name).filter(|value| !value.is_empty()));
    let code = locale.as_deref().and_then(|l| l.split(CODE_ENDS).next());
    if code == Some("fr") {
        Language::French
    } else {
        Language::English
    }
}

#[cfg(windows)]
pub fn current() -> Language {
    from_language_id(crate::os::ui_language())
}

#[cfg(not(windows))]
pub fn current() -> Language {
    from_locale(&crate::os::env)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    fn locale(variables: &[(&str, &str)]) -> Language {
        from_locale(&|name| {
            variables
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| String::from(*value))
        })
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
    fn lc_all_comes_before_lc_messages_and_lang() {
        let english_first = [
            ("LC_ALL", "C"),
            ("LC_MESSAGES", "fr_FR.UTF-8"),
            ("LANG", "fr_FR.UTF-8"),
        ];
        assert_eq!(locale(&english_first), Language::English);
        let french_first = [("LC_ALL", "fr_CA.UTF-8"), ("LANG", "en_US.UTF-8")];
        assert_eq!(locale(&french_first), Language::French);
    }

    #[test]
    fn lc_messages_comes_before_lang() {
        let variables = [("LC_MESSAGES", "fr_BE.UTF-8"), ("LANG", "en_GB.UTF-8")];
        assert_eq!(locale(&variables), Language::French);
    }

    #[test]
    fn empty_variables_are_skipped() {
        let variables = [("LC_ALL", ""), ("LC_MESSAGES", ""), ("LANG", "fr_FR.UTF-8")];
        assert_eq!(locale(&variables), Language::French);
    }

    #[test]
    fn only_the_fr_language_code_means_french() {
        for value in ["fr", "fr_FR", "fr.UTF-8", "fr@euro", "fr_CH.ISO-8859-1"] {
            assert_eq!(locale(&[("LANG", value)]), Language::French, "{}", value);
        }
        for value in [
            "fy_NL.UTF-8",
            "frr_DE.UTF-8",
            "C.UTF-8",
            "POSIX",
            "en_US.UTF-8",
        ] {
            assert_eq!(locale(&[("LANG", value)]), Language::English, "{}", value);
        }
    }

    #[test]
    fn no_locale_means_english() {
        assert_eq!(locale(&[]), Language::English);
    }
}
