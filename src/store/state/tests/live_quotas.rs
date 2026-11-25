use crate::session::Live;
use crate::state::State;

fn live_window(pct: f64) -> Option<(Option<Live>, Option<Live>)> {
    let window = Live {
        pct: Some(pct),
        at: Some(1_800_003_000.0),
    };
    Some((Some(window), None))
}

const STORED: &str = "q_at@a 1799999995
q5_at@a 1800003000
q5_pct@a 10
";

#[test]
fn a_live_render_with_the_same_values_leaves_the_state_alone() {
    let mut state = State::parse(STORED);
    state.quotas(live_window(10.0), 1_800_000_000.0, "a");
    assert!(!state.changed());
    assert_eq!(state.get("q_at@a"), Some("1799999995"));
}

#[test]
fn a_live_render_with_a_new_percentage_stamps_the_quotas() {
    let mut state = State::parse(STORED);
    state.quotas(live_window(11.0), 1_800_000_000.0, "a");
    assert_eq!(state.get("q_at@a"), Some("1800000000"));
    assert_eq!(state.get("q5_pct@a"), Some("11"));
}

#[test]
fn a_live_render_ten_seconds_later_stamps_the_quotas_again() {
    let mut state = State::parse(
        "q_at@a 1799999990
q5_at@a 1800003000
q5_pct@a 10
",
    );
    state.quotas(live_window(10.0), 1_800_000_000.0, "a");
    assert_eq!(state.get("q_at@a"), Some("1800000000"));
}

fn render(state: &mut State, pct: f64, at: f64) -> Option<f64> {
    let live = Live {
        pct: Some(pct),
        at: Some(at),
    };
    let quotas = state.quotas(Some((Some(live), None)), 1_800_000_000.0, "a");
    quotas.and_then(|q| q.five_hour).and_then(|q| q.pct)
}

#[test]
fn two_sessions_of_one_account_agree_on_the_highest_percentage() {
    let mut state = State::parse("");
    assert_eq!(render(&mut state, 46.0, 1_800_003_000.0), Some(46.0));
    let first = state.text();
    assert_eq!(render(&mut state, 29.0, 1_800_003_000.0), Some(46.0));
    assert_eq!(state.text(), first);
    assert_eq!(render(&mut state, 47.0, 1_800_003_020.0), Some(47.0));
    assert_eq!(state.get("q5_at@a"), Some("1800003000"));
}

#[test]
fn a_session_still_on_the_previous_window_does_not_overwrite_the_new_one() {
    let mut state = State::parse("");
    assert_eq!(render(&mut state, 5.0, 1_800_010_000.0), Some(5.0));
    assert_eq!(render(&mut state, 90.0, 1_800_001_000.0), Some(5.0));
    assert_eq!(state.get("q5_at@a"), Some("1800010000"));
    assert_eq!(render(&mut state, 2.0, 1_800_020_000.0), Some(2.0));
}

#[test]
fn a_reset_percentage_replaces_the_stored_one() {
    let mut state = State::parse("");
    assert_eq!(render(&mut state, 86.0, 1_800_003_000.0), Some(86.0));
    assert_eq!(render(&mut state, 0.0, 1_800_003_000.0), Some(0.0));
    assert_eq!(state.get("q5_pct@a"), Some("0"));
}

#[test]
fn an_absent_percentage_keeps_the_stored_one() {
    let mut state = State::parse(STORED);
    let live = Live {
        pct: None,
        at: Some(1_800_003_000.0),
    };
    let quotas = state.quotas(Some((Some(live), None)), 1_800_000_000.0, "a");
    assert_eq!(
        quotas.and_then(|q| q.five_hour).and_then(|q| q.pct),
        Some(10.0)
    );
    assert_eq!(state.get("q5_pct@a"), Some("10"));
}

fn render_both(state: &mut State, pct: f64) -> (Option<f64>, Option<f64>) {
    let window = |at| Live {
        pct: Some(pct),
        at: Some(at),
    };
    let live = (Some(window(1_800_003_000.0)), Some(window(1_800_500_000.0)));
    let quotas = state.quotas(Some(live), 1_800_000_000.0, "a").unwrap();
    let pct_of = |quota: Option<crate::render::Quota>| quota.and_then(|q| q.pct);
    (pct_of(quotas.five_hour), pct_of(quotas.seven_day))
}

#[test]
fn a_reset_to_a_small_percentage_in_the_same_window_replaces_the_stored_one() {
    let mut state = State::parse("");
    assert_eq!(render_both(&mut state, 86.0), (Some(86.0), Some(86.0)));
    assert_eq!(render_both(&mut state, 2.0), (Some(2.0), Some(2.0)));
    assert_eq!(state.get("q5_pct@a"), Some("2"));
    assert_eq!(state.get("q7_pct@a"), Some("2"));
}

#[test]
fn a_slightly_lower_percentage_from_another_session_keeps_the_highest() {
    let mut state = State::parse("");
    assert_eq!(render_both(&mut state, 40.0), (Some(40.0), Some(40.0)));
    assert_eq!(render_both(&mut state, 39.5), (Some(40.0), Some(40.0)));
}
