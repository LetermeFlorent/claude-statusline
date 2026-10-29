use super::*;

fn forced(value: &str) -> Language {
    let variable = |name: &str| (name == FORCED).then(|| String::from(value));
    choose(&variable, &|| Some(Language::English))
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

#[test]
fn each_language_names_its_code() {
    assert_eq!(Language::French.code(), "fr");
    assert_eq!(Language::English.code(), "en");
}
