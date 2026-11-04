use crate::render::Quota;
use crate::session::Live;
use crate::state::quotas::FIVE_HOUR_PERIOD;
use crate::state::State;

#[test]
fn quotas() {
    let now = 1_800_000_000.0;
    let mut state = State::parse("q_at@a 1799999999\nq5_at@a 1799990000\nq5_pct@a 40\n");
    let quotas = state.quotas(None, now, "a").unwrap();
    assert_eq!(
        quotas.five_hour,
        Some(Quota {
            pct: None,
            at: Some(1_800_008_000.0)
        })
    );
    assert_eq!(quotas.seven_day, None);
    assert!(state.quotas(None, now, "b").is_none());
    let mut state = State::parse("q_at@a 1\nq5_at@a 2000000\n");
    assert!(state.quotas(None, now, "a").is_none());
    let live = Some((
        Some(Live {
            pct: Some(10.5),
            at: Some(1791373904.7),
        }),
        None,
    ));
    let mut state = State::parse("q7_pct 3\nq7_pct@a 3\nq7_pct@b 4\nq7_x 2\nx 1\n");
    state.quotas(live, now, "a");
    assert_eq!(
        state.text(),
        "q5_at@a 1791373904.7\nq5_pct@a 10.5\nq7_pct@b 4\nq7_x 2\nq_at@a 1800000000\nx 1\n"
    );
}

#[test]
fn absurd_reset_times_are_dropped_without_hanging() {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut state =
            State::parse("q_at@claude 1800000000\nq5_at@claude -5e38\nq7_at@claude 1e300\n");
        let _ = sender.send(state.quotas(None, 1_800_000_100.0, "claude"));
    });
    let quotas = receiver.recv_timeout(std::time::Duration::from_secs(2));
    assert_eq!(quotas, Ok(None));
}

#[test]
fn a_live_quota_past_its_reset_loses_its_percentage() {
    let now = 1_800_000_000.0;
    let live = Live {
        pct: Some(85.0),
        at: Some(now - 600.0),
    };
    let mut state = State::parse("");
    let quotas = state.quotas(Some((Some(live), None)), now, "a").unwrap();
    let at = Some(now - 600.0 + FIVE_HOUR_PERIOD);
    assert_eq!(quotas.five_hour, Some(Quota { pct: None, at }));
    assert_eq!(state.get("q5_pct@a"), Some("85"));
}

#[test]
fn a_plan_without_a_five_hour_window_keeps_it_hidden() {
    let now = 1_800_000_000.0;
    let seven = Live {
        pct: Some(40.0),
        at: Some(now + 3600.0),
    };
    let mut state = State::parse("q5_at@a 1800001000\nq5_pct@a 9\n");
    let quotas = state.quotas(Some((None, Some(seven))), now, "a").unwrap();
    assert_eq!(quotas.five_hour, None);
    assert_eq!(state.get("q5_none@a"), Some("1"));
    assert_eq!(state.get("q5_pct@a"), None);
    let stored = state.quotas(None, now, "a").unwrap();
    assert_eq!(stored.five_hour, None);
    assert!(stored.seven_day.is_some());
    let mut state = State::parse("q_at@a 1799999999\nq7_at@a 1800003600\n");
    let idle = state.quotas(None, now, "a").unwrap();
    assert_eq!(idle.five_hour, Some(Quota::default()));
}

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
