use super::*;

const DARK: Decision = Decision {
    light: false,
    source: Source::Vscode,
};
const LIGHT: Decision = Decision {
    light: true,
    source: Source::System,
};
const ASSUMED: Decision = Decision {
    light: false,
    source: Source::Assumed,
};

fn stamp(n: usize) -> String {
    format!("{:016x}", n)
}

fn entries(state: &State) -> usize {
    state.with_prefix("bg@").count()
}

#[test]
fn two_projects_alternating_keep_both_entries() {
    let mut state = State::parse("");
    remember(&mut state, &stamp(1), DARK, 1000.0);
    remember(&mut state, &stamp(2), LIGHT, 1001.0);
    for round in 0..3 {
        let now = 1002.0 + round as f64;
        assert_eq!(recall(&mut state, &stamp(1), now), Some(DARK));
        assert_eq!(recall(&mut state, &stamp(2), now), Some(LIGHT));
    }
    assert_eq!(entries(&state), 2);
}

#[test]
fn a_fifth_stamp_evicts_the_one_stored_longest_ago() {
    let mut state = State::parse("");
    for n in 0..5 {
        remember(&mut state, &stamp(n), DARK, 1000.0 + n as f64);
    }
    assert_eq!(entries(&state), 4);
    assert_eq!(recall(&mut state, &stamp(0), 1010.0), None);
    assert_eq!(recall(&mut state, &stamp(4), 1010.0), Some(DARK));
}

#[test]
fn a_hit_after_an_hour_renews_the_entry() {
    let mut state = State::parse("");
    for n in 0..4 {
        remember(&mut state, &stamp(n), DARK, 1000.0 + n as f64);
    }
    recall(&mut state, &stamp(0), 9000.0);
    remember(&mut state, &stamp(4), DARK, 9001.0);
    assert_eq!(recall(&mut state, &stamp(0), 9002.0), Some(DARK));
    assert_eq!(recall(&mut state, &stamp(1), 9002.0), None);
}

#[test]
fn a_quick_hit_writes_nothing() {
    let mut state = State::parse("");
    remember(&mut state, &stamp(1), ASSUMED, 1000.0);
    let mut again = State::parse(&state.text());
    assert_eq!(recall(&mut again, &stamp(1), 1100.0), Some(ASSUMED));
    assert!(!again.changed());
}

#[test]
fn the_source_name_may_hold_spaces() {
    let mut state = State::parse("");
    remember(&mut state, &stamp(1), ASSUMED, 1000.0);
    let reloaded = State::parse(&state.text());
    let mut reloaded = reloaded;
    assert_eq!(recall(&mut reloaded, &stamp(1), 1001.0), Some(ASSUMED));
}

#[test]
fn the_single_entry_keys_of_older_versions_are_removed() {
    let mut state = State::parse("bg_stamp x\nbg_host dark\nbg_source zed\nbg_at 5\nz 1\n");
    drop_legacy(&mut state);
    assert_eq!(state.text(), "z 1\n");
}

#[test]
fn malformed_entries_are_ignored_and_dropped() {
    let bad = [
        "x",
        "1 dark",
        "a dark zed",
        "5 grey zed",
        "5 dark ",
        "5  dark zed",
    ];
    for value in bad {
        let mut state = State::parse(&format!("bg@{} {}\n", stamp(1), value));
        assert_eq!(recall(&mut state, &stamp(1), 1000.0), None, "{}", value);
        remember(&mut state, &stamp(2), DARK, 1001.0);
        assert_eq!(entries(&state), 1, "{}", value);
        assert!(state.get(&format!("bg@{}", stamp(1))).is_none());
    }
}

#[test]
fn an_evicted_entry_does_not_come_back_from_the_file() {
    let mut mine = State::parse("");
    for n in 0..4 {
        remember(&mut mine, &stamp(n), DARK, 1000.0 + n as f64);
    }
    let mut saved = State::parse(&mine.text());
    let mut mine = State::parse(&mine.text());
    remember(&mut mine, &stamp(4), DARK, 1100.0);
    saved.apply(&mine.changes());
    assert_eq!(entries(&saved), 4);
    assert_eq!(recall(&mut saved, &stamp(0), 1101.0), None);
}
