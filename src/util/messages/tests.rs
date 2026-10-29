use super::*;

const ALL: [&Text; 48] = [
    &BENCH_SUMMARY,
    &BENCH_STATE,
    &BENCH_CONFIG,
    &BENCH_SESSION,
    &BENCH_THEME,
    &BENCH_COMPACT,
    &BENCH_SYSTEM,
    &BENCH_RENDER,
    &BENCH_WRITE,
    &SEVEN_DAY_LABEL,
    &SPEND_LABEL,
    &COLD_CACHE_LABEL,
    &UPDATE_LABEL,
    &RESET_DAYS,
    &RESET_HOURS,
    &RESET_MINUTES,
    &RAM_USAGE,
    &DECIMAL_POINT,
    &UNIT_BILLION,
    &UNKNOWN_ARGUMENT,
    &BAD_BENCH_COUNT,
    &SETTINGS_UNREADABLE,
    &SETTINGS_INVALID,
    &CANNOT_WRITE,
    &BACKUP,
    &GLOBAL_REPAIRED,
    &GLOBAL_KEPT,
    &NO_HOME,
    &NO_EXE,
    &INSTALLED,
    &CONFIG_WRITTEN,
    &RESTART,
    &NO_SETTINGS,
    &NOTHING_TO_REMOVE,
    &REMOVED,
    &NO_RELEASE,
    &CURL_MISSING,
    &RATE_LIMITED,
    &NO_ASSET,
    &UPDATE_FAILED,
    &UP_TO_DATE,
    &UPDATED,
    &STATE_DIR_FAILED,
    &NO_STATE_DIR,
    &UPDATE_RUNNING,
    &PANICKED,
    &NOT_RESTORED,
    &WRONG_VERSION,
];

#[test]
fn values_fill_the_placeholders_in_order() {
    assert_eq!(
        fill("{} -> {}\n", &["2.1.0", "v2.2.0"]),
        "2.1.0 -> v2.2.0\n"
    );
}

#[test]
fn braces_inside_a_value_stay_as_they_are() {
    assert_eq!(fill("{} and {}", &["C:/{}", "x"]), "C:/{} and x");
}

#[test]
fn a_missing_value_leaves_its_placeholder_empty() {
    assert_eq!(fill("a{}b{}c", &["1"]), "a1bc");
}

#[test]
fn each_language_gets_its_own_text() {
    let text = Text::new("up to date", "à jour");
    assert_eq!(text.in_language(Language::English), "up to date");
    assert_eq!(text.in_language(Language::French), "à jour");
}

#[test]
fn the_panic_line_names_the_file_line_and_column() {
    let location = core::panic::Location::caller();
    let mut line = String::new();
    PANICKED.write_to(&mut line, location).unwrap();
    let place = format!(
        "{}:{}:{}",
        location.file(),
        location.line(),
        location.column()
    );
    assert_eq!(line, PANICKED.with(&[&place]));
    assert!(line.contains(&place), "{}", line);
    assert_eq!(line.lines().count(), 1, "{}", line);
}

#[test]
fn the_help_names_the_account_folder() {
    for help in [HELP.english, HELP.french] {
        assert_eq!(help.matches("$CLAUDE_CONFIG_DIR").count(), 2, "{}", help);
    }
}

#[test]
fn both_languages_take_the_same_values_and_line_ending() {
    let placeholders = |s: &str| s.matches(PLACEHOLDER).count();
    for text in ALL.into_iter().chain(CHECK_TEXTS).chain([&HELP]) {
        assert!(!text.english.is_empty(), "{}", text.french);
        assert!(!text.french.is_empty(), "{}", text.english);
        assert_eq!(
            placeholders(text.english),
            placeholders(text.french),
            "{}",
            text.english
        );
        assert_eq!(
            text.english.ends_with('\n'),
            text.french.ends_with('\n'),
            "{}",
            text.english
        );
    }
}
