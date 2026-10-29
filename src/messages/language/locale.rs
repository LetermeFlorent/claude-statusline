use super::language::{from_tag, Language, CODE_ENDS};
use alloc::string::String;

const VARIABLES: [&str; 3] = ["LC_ALL", "LC_MESSAGES", "LANG"];
const PREFERENCES: &str = "LANGUAGE";
const PREFERENCE_SEPARATOR: char = ':';
const NEUTRAL: [&str; 2] = ["C", "POSIX"];

fn neutral(locale: &str) -> bool {
    let code = locale.split(CODE_ENDS).next().unwrap_or_default();
    NEUTRAL.iter().any(|name| code.eq_ignore_ascii_case(name))
}

pub fn from_locale(variable: &dyn Fn(&str) -> Option<String>) -> Option<Language> {
    let locale = VARIABLES
        .iter()
        .find_map(|name| variable(name).filter(|value| !value.is_empty()));
    if locale.as_deref().is_some_and(neutral) {
        return Some(Language::English);
    }
    variable(PREFERENCES)
        .and_then(|list| list.split(PREFERENCE_SEPARATOR).find_map(from_tag))
        .or_else(|| locale.map(|l| from_tag(&l).unwrap_or(Language::English)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locale(variables: &[(&str, &str)]) -> Option<Language> {
        from_locale(&|name| {
            variables
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| String::from(*value))
        })
    }

    #[test]
    fn lc_all_comes_before_lc_messages_and_lang() {
        let english_first = [
            ("LC_ALL", "C"),
            ("LC_MESSAGES", "fr_FR.UTF-8"),
            ("LANG", "fr_FR.UTF-8"),
        ];
        assert_eq!(locale(&english_first), Some(Language::English));
        let french_first = [("LC_ALL", "fr_CA.UTF-8"), ("LANG", "en_US.UTF-8")];
        assert_eq!(locale(&french_first), Some(Language::French));
    }

    #[test]
    fn lc_messages_comes_before_lang() {
        let variables = [("LC_MESSAGES", "fr_BE.UTF-8"), ("LANG", "en_GB.UTF-8")];
        assert_eq!(locale(&variables), Some(Language::French));
    }

    #[test]
    fn empty_variables_are_skipped() {
        let variables = [("LC_ALL", ""), ("LC_MESSAGES", ""), ("LANG", "fr_FR.UTF-8")];
        assert_eq!(locale(&variables), Some(Language::French));
    }

    #[test]
    fn only_the_fr_language_code_means_french() {
        for value in ["fr", "fr_FR", "fr.UTF-8", "fr@euro", "fr_CH.ISO-8859-1"] {
            let found = locale(&[("LANG", value)]);
            assert_eq!(found, Some(Language::French), "{}", value);
        }
        for value in [
            "fy_NL.UTF-8",
            "frr_DE.UTF-8",
            "C.UTF-8",
            "POSIX",
            "en_US.UTF-8",
        ] {
            let found = locale(&[("LANG", value)]);
            assert_eq!(found, Some(Language::English), "{}", value);
        }
    }

    #[test]
    fn language_comes_before_the_locale_variables() {
        let variables = [("LANGUAGE", "fr:en"), ("LANG", "en_US.UTF-8")];
        assert_eq!(locale(&variables), Some(Language::French));
        let variables = [("LANGUAGE", "en:fr"), ("LANG", "fr_FR.UTF-8")];
        assert_eq!(locale(&variables), Some(Language::English));
    }

    #[test]
    fn language_skips_entries_the_program_has_no_text_for() {
        let variables = [("LANGUAGE", "de:fr:en"), ("LANG", "en_US.UTF-8")];
        assert_eq!(locale(&variables), Some(Language::French));
        let variables = [("LANGUAGE", ":fr_CA"), ("LANG", "en_US.UTF-8")];
        assert_eq!(locale(&variables), Some(Language::French));
    }

    #[test]
    fn language_alone_counts_when_no_locale_is_set() {
        assert_eq!(locale(&[("LANGUAGE", "fr")]), Some(Language::French));
    }

    #[test]
    fn a_neutral_locale_ignores_language() {
        for neutral in ["C", "POSIX", "C.UTF-8"] {
            let variables = [("LANGUAGE", "fr"), ("LANG", neutral)];
            assert_eq!(locale(&variables), Some(Language::English), "{}", neutral);
        }
    }

    #[test]
    fn no_locale_at_all_gives_no_answer() {
        assert_eq!(locale(&[]), None);
        assert_eq!(locale(&[("LANG", ""), ("LANGUAGE", "")]), None);
    }
}
