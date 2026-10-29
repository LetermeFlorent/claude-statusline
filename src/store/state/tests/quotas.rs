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
