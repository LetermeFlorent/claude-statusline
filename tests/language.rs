use std::process::Command;

const FRENCH_MARK: &str = "Sans argument";
const ENGLISH_MARK: &str = "With no argument";
const LOCALE_VARIABLES: [&str; 5] = [
    "STATUSLINE_LANG",
    "LANGUAGE",
    "LC_ALL",
    "LC_MESSAGES",
    "LANG",
];

fn help(variables: &[(&str, &str)]) -> String {
    let mut command = Command::new(env!("CARGO_BIN_EXE_statusline"));
    for name in LOCALE_VARIABLES {
        command.env_remove(name);
    }
    let output = command.arg("--help").envs(variables.iter().copied());
    String::from_utf8(output.output().unwrap().stdout).unwrap()
}

#[test]
fn statusline_lang_fr_prints_the_french_help() {
    let text = help(&[("STATUSLINE_LANG", "fr")]);
    assert!(text.contains(FRENCH_MARK), "{}", text);
}

#[test]
fn statusline_lang_en_prints_the_english_help() {
    let text = help(&[("STATUSLINE_LANG", "en")]);
    assert!(text.contains(ENGLISH_MARK), "{}", text);
}

#[test]
fn statusline_lang_beats_a_french_locale() {
    let text = help(&[("STATUSLINE_LANG", "en"), ("LANG", "fr_FR.UTF-8")]);
    assert!(text.contains(ENGLISH_MARK), "{}", text);
}

#[cfg(unix)]
#[test]
fn a_french_locale_prints_the_french_help() {
    let text = help(&[("LANG", "fr_FR.UTF-8")]);
    assert!(text.contains(FRENCH_MARK), "{}", text);
}

#[cfg(unix)]
#[test]
fn the_c_locale_prints_the_english_help() {
    let text = help(&[("LANG", "C")]);
    assert!(text.contains(ENGLISH_MARK), "{}", text);
}

#[cfg(unix)]
#[test]
fn language_lists_the_first_supported_language() {
    let text = help(&[("LANGUAGE", "de:fr:en"), ("LANG", "en_US.UTF-8")]);
    assert!(text.contains(FRENCH_MARK), "{}", text);
}

fn language(variables: &[(&str, &str)]) -> (String, Option<i32>) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_statusline"));
    for name in LOCALE_VARIABLES {
        command.env_remove(name);
    }
    let output = command.arg("--language").envs(variables.iter().copied());
    let output = output.output().unwrap();
    (
        String::from_utf8(output.stdout).unwrap(),
        output.status.code(),
    )
}

#[test]
fn language_prints_the_code_that_was_forced() {
    assert_eq!(
        language(&[("STATUSLINE_LANG", "fr")]),
        ("fr\n".into(), Some(0))
    );
    assert_eq!(
        language(&[("STATUSLINE_LANG", "en")]),
        ("en\n".into(), Some(0))
    );
}

#[cfg(unix)]
#[test]
fn language_prints_the_code_of_the_locale() {
    assert_eq!(
        language(&[("LANG", "fr_FR.UTF-8")]),
        ("fr\n".into(), Some(0))
    );
    assert_eq!(language(&[("LANG", "C")]), ("en\n".into(), Some(0)));
}
