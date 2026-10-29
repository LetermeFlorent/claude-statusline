use crate::state::State;

#[test]
fn lines_come_back_sorted_and_changes_are_tracked() {
    let mut state = State::parse("z 1\r\nq5_at  12\na b c\n");
    assert_eq!(state.get("a"), Some("b c"));
    assert_eq!(state.num("q5_at"), None);
    assert!(!state.changed());
    state.set("b", "2");
    assert_eq!(state.text(), "a b c\nb 2\nq5_at  12\nz 1\n");
    assert!(state.changed());
}

#[test]
fn saving_keeps_what_another_process_wrote_since_loading() {
    let mut mine = State::parse("bg_stamp 1\nq5_pct@a 10\nq_at@a 5\n");
    mine.set("q5_pct@a", "20");
    mine.remove("q_at@a");
    mine.set("cpu_at", "7");
    let mut saved = State::parse("bg_stamp 2\nq5_pct@a 10\nq5_pct@b 30\nq_at@a 5\n");
    saved.apply(&mine.changes());
    assert_eq!(
        saved.text(),
        "bg_stamp 2\ncpu_at 7\nq5_pct@a 20\nq5_pct@b 30\n"
    );
    assert!(saved.changed());
}

#[test]
fn a_state_set_back_to_what_it_loaded_changes_nothing_on_disk() {
    let mut mine = State::parse("q_at@a 5\n");
    mine.remove("q_at@a");
    mine.set("q_at@a", "5");
    let mut saved = State::parse("q_at@a 6\n");
    saved.apply(&mine.changes());
    assert_eq!(saved.text(), "q_at@a 6\n");
    assert!(!saved.changed());
}

#[test]
fn keys_are_listed_by_prefix() {
    let state = State::parse("a@1 x\na@2 y\nb@1 z\n");
    let listed: Vec<_> = state.with_prefix("a@").collect();
    assert_eq!(listed, [("a@1", "x"), ("a@2", "y")]);
}

#[test]
fn values_cannot_add_lines() {
    let mut state = State::parse("");
    state.set("bg_stamp", "x\nq5_at@claude -5e38");
    state.set("bg_host", "dark\r");
    state.set("bad key", "1");
    state.set("nul\0", "1");
    state.set("ok", "a b");
    assert_eq!(State::parse(&state.text()).text(), "ok a b\n");
}

#[test]
fn a_byte_order_mark_at_the_start_of_the_file_is_ignored() {
    let state = State::parse("\u{feff}q_at@a 5\nz 1\n");
    assert_eq!(state.get("q_at@a"), Some("5"));
    assert_eq!(state.text(), "q_at@a 5\nz 1\n");
}

#[test]
fn non_finite_numbers_read_as_missing() {
    let state = State::parse("a inf\nb NaN\nc 1e999\nd 2.5\n");
    let read = ["a", "b", "c", "d"].map(|k| state.num(k));
    assert_eq!(read, [None, None, None, Some(2.5)]);
}
