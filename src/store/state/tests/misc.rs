use crate::state::{account_tag, recent, State};

#[test]
fn account_tag_comes_from_the_config_folder() {
    assert_eq!(account_tag("C:\\Users\\u\\.claude"), "claude");
    assert_eq!(account_tag("/home/u/.claude-compte2/"), "claude-compte2");
    assert_eq!(account_tag("/home/u/.Claude Work/"), "claude_work");
    assert_eq!(account_tag(""), "claude");
}

#[test]
fn times_stamped_in_the_future_are_not_recent() {
    assert!(recent(100.0, 129.0, 30.0));
    assert!(!recent(100.0, 130.0, 30.0));
    assert!(!recent(100.0, 99.0, 30.0));
    let mut state = State::parse("q_at@a 1800000600\nq5_at@a 1800010000\nq5_pct@a 40\n");
    assert!(state.quotas(None, 1_800_000_000.0, "a").is_none());
}
