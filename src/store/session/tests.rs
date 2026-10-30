use super::*;

#[test]
fn the_spend_limit_is_read_next_to_the_windows() {
    let text = "{\"rate_limits\":{\"spend_limit\":{\"used_percentage\":63.5,\"resets_at\":1800000000,\"used_usd\":3.1}}}";
    let session = Session::parse(text);
    assert!(session.live.is_none());
    let spend = session.spend.unwrap();
    assert_eq!((spend.pct, spend.at), (Some(63.5), Some(1.8e9)));
    assert!(Session::parse("{\"rate_limits\":{\"spend_limit\":null}}")
        .spend
        .is_none());
}

#[test]
fn the_prompt_cache_warmth_is_read_when_it_is_a_boolean() {
    let warm = |text: &str| Session::parse(text).cache_warm;
    assert_eq!(warm("{\"prompt_cache\":{\"warm\":true}}"), Some(true));
    assert_eq!(warm("{\"prompt_cache\":{\"warm\":false}}"), Some(false));
    assert_eq!(warm("{\"prompt_cache\":{\"warm\":1}}"), None);
    assert_eq!(warm("{\"prompt_cache\":{}}"), None);
    assert_eq!(warm("{}"), None);
}

#[test]
fn reads_model_effort_tokens_and_rate_limits() {
    let session = Session::parse(
        "{\"model\":{\"display_name\":\"Opus 4 (1M context)\"},\"effort\":{\"level\":\" high \"}}",
    );
    assert_eq!(session.model, "Opus 4");
    assert_eq!(session.effort.as_deref(), Some("high"));
    assert_eq!(
        Session::parse("{\"model\":{\"display_name\":\"(x)\",\"id\":\"i\"}}").model,
        ""
    );
    assert_eq!(
        Session::parse("{\"model\":{\"display_name\":\"Unknown\",\"id\":\"i\"}}").model,
        "i"
    );
    assert_eq!(Session::parse("{\"model\":\"Opus\"}").model, "claude");
    let tokens_for = |cu: &str| {
        Session::parse(&alloc::format!(
            "{{\"context_window\":{{\"current_usage\":{}}}}}",
            cu
        ))
        .ctx
        .tokens
    };
    assert_eq!(
        tokens_for("{\"input_tokens\":1000.0,\"cache_read_input_tokens\":1.5e3}"),
        Some(2500)
    );
    assert_eq!(tokens_for("{\"input_tokens\":1000.7}"), None);
    assert_eq!(tokens_for("{\"input_tokens\":null}"), None);
    assert_eq!(tokens_for("[1]"), Some(0));
    assert_eq!(tokens_for("null"), None);
    assert!(Session::parse("{\"rate_limits\":{}}").live.is_none());
    assert!(Session::parse("{\"rate_limits\":{}}").spend.is_none());
    assert!(Session::parse("{\"rate_limits\":{\"five_hour\":null}}")
        .live
        .is_some());
}

fn tokens_of(context_window: &str) -> Option<u64> {
    let text = alloc::format!("{{\"context_window\":{}}}", context_window);
    Session::parse(&text).ctx.tokens
}

#[test]
fn total_input_tokens_fills_in_when_current_usage_is_null() {
    let got = tokens_of("{\"current_usage\":null,\"total_input_tokens\":42000}");
    assert_eq!(got, Some(42_000));
}

#[test]
fn total_input_tokens_fills_in_when_current_usage_is_missing() {
    assert_eq!(tokens_of("{\"total_input_tokens\":7}"), Some(7));
}

#[test]
fn current_usage_wins_over_total_input_tokens() {
    let got = tokens_of("{\"current_usage\":{\"input_tokens\":5},\"total_input_tokens\":9}");
    assert_eq!(got, Some(5));
}

#[test]
fn unusable_current_usage_does_not_fall_back_to_the_total() {
    let got = tokens_of("{\"current_usage\":{\"input_tokens\":1.5},\"total_input_tokens\":9}");
    assert_eq!(got, None);
}

#[test]
fn invalid_total_input_tokens_is_ignored() {
    assert_eq!(tokens_of("{\"total_input_tokens\":-3}"), None);
    assert_eq!(tokens_of("{\"total_input_tokens\":2.5}"), None);
    assert_eq!(tokens_of("{\"total_input_tokens\":\"9\"}"), None);
}

#[test]
fn terminal_controls_are_dropped_from_model_and_effort() {
    let session = Session::parse(
        r#"{"model":{"display_name":"Opus\u001b]52;c;aGk=\u0007"},"effort":{"level":"hi\u009bgh\u007f"}}"#,
    );
    assert_eq!(session.model, "Opus]52;c;aGk=");
    assert_eq!(session.effort.as_deref(), Some("high"));
}

#[test]
fn bidi_controls_are_dropped() {
    let session = Session::parse(r#"{"model":{"id":"\u2066claude\u202e-opus\u2069"}}"#);
    assert_eq!(session.model, "claude-opus");
}

#[test]
fn long_model_names_are_capped() {
    let name = "x".repeat(10_000);
    let session = Session::parse(&alloc::format!(
        r#"{{"model":{{"display_name":"{}"}}}}"#,
        name
    ));
    assert_eq!(
        session.model.chars().count(),
        crate::render::MAX_LABEL_CHARS
    );
}
