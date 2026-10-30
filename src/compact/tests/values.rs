use super::*;

#[test]
fn reads_the_plain_window_setting() {
    let text = plain("300000");
    let got = find(&inputs(None, Some(BIG)), &[("a", &text)]);
    assert_eq!(got, Some(300_000));
}

#[test]
fn model_settings_window_wins_over_the_plain_one() {
    let text = format!(
        "{{\"autoCompactWindow\": 300000, \"modelSettings\": {{\"{}\": {{\"autoCompactWindow\": 250000}}}}}}",
        MODEL
    );
    let got = find(&inputs(Some(MODEL), Some(BIG)), &[("a", &text)]);
    assert_eq!(got, Some(250_000));
}

#[test]
fn model_settings_of_another_model_are_ignored() {
    let text = per_model("claude-haiku-5", "250000");
    let got = find(&inputs(Some(MODEL), Some(BIG)), &[("a", &text)]);
    assert_eq!(got, None);
}

#[test]
fn k_suffix_is_read_in_settings() {
    let text = plain("\"200k\"");
    let got = find(&inputs(None, Some(BIG)), &[("a", &text)]);
    assert_eq!(got, Some(200_000));
}

#[test]
fn m_suffix_is_read_in_settings() {
    let text = per_model(MODEL, "\"1M\"");
    let got = find(&inputs(Some(MODEL), Some(BIG)), &[("a", &text)]);
    assert_eq!(got, Some(1_000_000));
}

#[test]
fn suffix_is_read_in_the_environment_variable() {
    let mut given = inputs(None, Some(BIG));
    given.env = Some(" 300k ");
    assert_eq!(find(&given, &[]), Some(300_000));
}

#[test]
fn window_below_the_minimum_is_raised() {
    let text = plain("50000");
    let got = find(&inputs(None, Some(BIG)), &[("a", &text)]);
    assert_eq!(got, Some(100_000));
}

#[test]
fn window_above_the_maximum_is_lowered() {
    let text = plain("\"5M\"");
    let got = find(&inputs(None, Some(5_000_000.0)), &[("a", &text)]);
    assert_eq!(got, Some(1_000_000));
}

#[test]
fn window_is_clamped_to_the_model_context_window() {
    let text = plain("500000");
    let got = find(&inputs(None, Some(200_000.0)), &[("a", &text)]);
    assert_eq!(got, Some(200_000));
}
