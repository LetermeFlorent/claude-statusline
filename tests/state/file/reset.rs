use super::{input, now, render, FIVE_HOURS, SEVEN_DAYS};
use crate::common::Home;

fn shown(home: &Home, lang: &str, percent: &str, reset: u64) -> String {
    let mut command = home.session_in(".claude");
    command.env("STATUSLINE_LANG", lang);
    render(command, &input_with(percent, reset))
}

fn input_with(percent: &str, reset: u64) -> String {
    input((0, 0), (41, now() + SEVEN_DAYS)).replace(
        "\"five_hour\":{\"used_percentage\":0,\"resets_at\":0}",
        &format!(
            "\"five_hour\":{{\"used_percentage\":{},\"resets_at\":{}}}",
            percent, reset
        ),
    )
}

fn five_hour(text: &str) -> String {
    let segment = text
        .split(" | ")
        .find(|part| part.starts_with("5h "))
        .unwrap();
    segment
        .split(' ')
        .find(|word| word.ends_with('%'))
        .unwrap()
        .to_string()
}

#[test]
fn a_reset_window_shows_zero_not_the_stored_value() {
    let home = Home::new("reset-zero");
    let window = now() + FIVE_HOURS;
    assert_eq!(five_hour(&shown(&home, "en", "86", window)), "86%");
    assert_eq!(five_hour(&shown(&home, "en", "0", window)), "0%");
    assert_eq!(five_hour(&shown(&home, "fr", "0", window)), "0%");
}

#[test]
fn a_fractional_percentage_shows_one_decimal() {
    let home = Home::new("reset-fraction");
    let text = shown(&home, "en", "12.34", now() + FIVE_HOURS);
    assert_eq!(five_hour(&text), "12.3%");
    let text = shown(&home, "fr", "25.3", now() + FIVE_HOURS);
    assert_eq!(five_hour(&text), "25,3%");
}

#[test]
fn a_reset_to_a_small_percentage_in_the_same_windows_shows_the_new_value() {
    let home = Home::new("reset-small");
    let start = now();
    let shown = |percent| {
        let mut command = home.session_in(".claude");
        command.env("STATUSLINE_LANG", "en");
        let windows = (percent, start + FIVE_HOURS);
        render(command, &input(windows, (percent, start + SEVEN_DAYS)))
    };
    let percents = |text: &str| -> Vec<String> {
        text.split(" | ")
            .filter(|part| part.starts_with("5h ") || part.starts_with("7d "))
            .filter_map(|part| part.split(' ').find(|word| word.ends_with('%')))
            .map(String::from)
            .collect()
    };
    assert_eq!(percents(&shown(86)), ["86%", "86%"]);
    assert_eq!(percents(&shown(2)), ["2%", "2%"]);
}
