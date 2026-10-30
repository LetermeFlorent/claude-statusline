use super::*;

pub(super) const MODEL: &str = "claude-opus-5";
pub(super) const BIG: f64 = 1_000_000.0;

pub(super) fn files() -> [String; 3] {
    [String::from("a"), String::from("b"), String::from("c")]
}

pub(super) fn inputs<'a>(model: Option<&'a str>, size: Option<f64>) -> Inputs<'a> {
    Inputs {
        env: None,
        disable: None,
        model,
        size,
    }
}

pub(super) fn find(inputs: &Inputs, contents: &[(&str, &str)]) -> Option<u64> {
    let read = |f: &str| {
        contents
            .iter()
            .find(|(name, _)| *name == f)
            .map(|(_, text)| String::from(*text))
    };
    let stat = |f: &str| read(f).map(|text| (text.len() as u64, 1));
    let mut state = State::parse("");
    window(inputs, &files(), &mut state, 1000.0, &disk(&stat, &read))
}

pub(super) fn disk<'a>(
    stat: &'a dyn Fn(&str) -> Option<(u64, i64)>,
    read: &'a dyn Fn(&str) -> Option<String>,
) -> Disk<'a> {
    Disk { stat, read }
}

pub(super) fn plain(value: &str) -> String {
    format!("{{\"autoCompactWindow\": {}}}", value)
}

pub(super) fn per_model(model: &str, value: &str) -> String {
    format!(
        "{{\"modelSettings\": {{\"{}\": {{\"autoCompactWindow\": {}}}}}}}",
        model, value
    )
}

mod cache;
mod folder;
mod saved;
mod switches;
mod values;

#[test]
fn cache_is_reused_for_thirty_seconds_then_checked_by_stamp() {
    let text = plain("300000");
    let read = |f: &str| (f == "a").then(|| text.clone());
    let stat = |f: &str| (f == "a").then_some((10, 20));
    let given = inputs(None, Some(BIG));
    let mut state = State::parse("");
    let first = window(&given, &files(), &mut state, 1000.0, &disk(&stat, &read));
    assert_eq!(first, Some(300_000));
    let no_read = |_: &str| -> Option<String> { panic!() };
    let soon = window(&given, &files(), &mut state, 1010.0, &disk(&stat, &no_read));
    assert_eq!(soon, Some(300_000));
    let later = window(&given, &files(), &mut state, 2000.0, &disk(&stat, &no_read));
    assert_eq!(later, Some(300_000));
    let moved = |f: &str| (f == "a").then_some((11, 20));
    let other = String::from("{\"autoCompactWindow\": 400000}");
    let read_new = |_: &str| Some(other.clone());
    let fresh = window(
        &given,
        &files(),
        &mut state,
        3000.0,
        &disk(&moved, &read_new),
    );
    assert_eq!(fresh, Some(400_000));
}

#[test]
fn another_model_does_not_reuse_the_cache() {
    let text = per_model(MODEL, "250000");
    let read = |f: &str| (f == "a").then(|| text.clone());
    let stat = |f: &str| (f == "a").then_some((10, 20));
    let mut state = State::parse("");
    let first = inputs(Some(MODEL), Some(BIG));
    let got = window(&first, &files(), &mut state, 1000.0, &disk(&stat, &read));
    assert_eq!(got, Some(250_000));
    let second = inputs(Some("claude-haiku-5"), Some(BIG));
    let got = window(&second, &files(), &mut state, 1001.0, &disk(&stat, &read));
    assert_eq!(got, None);
}

#[test]
fn stamp_lists_size_and_date_of_each_file() {
    let text = plain("300000");
    let read = |f: &str| (f == "a").then(|| text.clone());
    let stat = |f: &str| if f == "c" { None } else { Some((10, 20)) };
    let mut state = State::parse("");
    let given = inputs(None, Some(BIG));
    window(&given, &files(), &mut state, 1000.0, &disk(&stat, &read));
    let stored = state.with_prefix("compact@").next().map(|(_, v)| v);
    assert_eq!(stored, Some("1000 10:20;10:20;0; 300000"));
}
