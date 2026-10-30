use super::*;

#[test]
fn disable_auto_compact_variable_removes_the_window() {
    let text = plain("300000");
    let mut given = inputs(None, Some(BIG));
    given.disable = Some("1");
    assert_eq!(find(&given, &[("a", &text)]), None);
    given.disable = Some("0");
    assert_eq!(find(&given, &[("a", &text)]), Some(300_000));
}

#[test]
fn disabled_account_file_beats_a_project_window() {
    let off = String::from("{\"autoCompactEnabled\": false}");
    let text = plain("300000");
    let got = find(&inputs(None, Some(BIG)), &[("a", &off), ("b", &text)]);
    assert_eq!(got, None);
}

#[test]
fn disabled_setting_beats_the_environment_variable() {
    let off = String::from("{\"autoCompactEnabled\": false}");
    let mut given = inputs(None, Some(BIG));
    given.env = Some("300000");
    assert_eq!(find(&given, &[("a", &off)]), None);
}

#[test]
fn later_file_can_enable_again() {
    let off = String::from("{\"autoCompactEnabled\": false}");
    let on = String::from("{\"autoCompactEnabled\": true, \"autoCompactWindow\": 300000}");
    let got = find(&inputs(None, Some(BIG)), &[("a", &off), ("b", &on)]);
    assert_eq!(got, Some(300_000));
}

#[test]
fn each_key_resolves_on_its_own_across_files() {
    let account = per_model(MODEL, "250000");
    let project = plain("400000");
    let got = find(
        &inputs(Some(MODEL), Some(BIG)),
        &[("a", &account), ("b", &project)],
    );
    assert_eq!(got, Some(250_000));
}

#[test]
fn invalid_value_in_a_later_file_keeps_the_earlier_one() {
    let account = plain("300000");
    let project = plain("\"soon\"");
    let got = find(
        &inputs(None, Some(BIG)),
        &[("a", &account), ("b", &project)],
    );
    assert_eq!(got, Some(300_000));
}

#[test]
fn environment_variable_wins_over_the_files() {
    let text = plain("300000");
    let mut given = inputs(None, Some(BIG));
    given.env = Some("200000");
    assert_eq!(find(&given, &[("a", &text)]), Some(200_000));
    given.env = Some("1.5");
    assert_eq!(find(&given, &[("a", &text)]), Some(300_000));
}
