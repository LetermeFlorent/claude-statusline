use super::*;
use core::cell::Cell;

const SLOTS: usize = 4;

fn project(n: usize) -> [String; 3] {
    [1, 2, 3].map(|part| format!("project{}/file{}", n, part))
}

fn key_of(files: &[String; 3]) -> String {
    let parts = files.iter().map(String::as_str);
    format!("compact@{}", fingerprint(parts))
}

fn text_of(n: usize) -> String {
    plain(&format!("{}", 200_000 + 10_000 * n))
}

fn render(state: &mut State, n: usize, now: f64, reads: &Cell<usize>) -> Option<u64> {
    let text = text_of(n);
    let read = |f: &str| {
        reads.set(reads.get() + 1);
        f.ends_with("file1").then(|| text.clone())
    };
    let stat = |f: &str| f.ends_with("file1").then_some((10, 20));
    let given = inputs(None, Some(BIG));
    window(&given, &project(n), state, now, &stat, &read)
}

fn entries(state: &State) -> usize {
    state.with_prefix("compact@").count()
}

#[test]
fn two_projects_alternating_find_their_cache_each_time() {
    let mut state = State::parse("");
    let reads = Cell::new(0);
    assert_eq!(render(&mut state, 0, 1000.0, &reads), Some(200_000));
    assert_eq!(render(&mut state, 1, 1001.0, &reads), Some(210_000));
    let scanned = reads.get();
    assert_eq!(render(&mut state, 0, 1002.0, &reads), Some(200_000));
    assert_eq!(render(&mut state, 1, 1003.0, &reads), Some(210_000));
    assert_eq!(render(&mut state, 0, 2000.0, &reads), Some(200_000));
    assert_eq!(render(&mut state, 1, 2001.0, &reads), Some(210_000));
    assert_eq!(reads.get(), scanned);
    assert_eq!(entries(&state), 2);
}

#[test]
fn a_fifth_project_evicts_the_one_checked_longest_ago() {
    let mut state = State::parse("");
    let reads = Cell::new(0);
    for n in 0..=SLOTS {
        render(&mut state, n, 1000.0 + n as f64, &reads);
    }
    assert_eq!(entries(&state), SLOTS);
    assert!(state.get(&key_of(&project(0))).is_none());
    for n in 1..=SLOTS {
        assert!(state.get(&key_of(&project(n))).is_some());
    }
}

#[test]
fn a_project_checked_again_is_kept_over_an_older_one() {
    let mut state = State::parse("");
    let reads = Cell::new(0);
    for n in 0..SLOTS {
        render(&mut state, n, 1000.0 + n as f64, &reads);
    }
    render(&mut state, 0, 2000.0, &reads);
    render(&mut state, SLOTS, 2001.0, &reads);
    assert!(state.get(&key_of(&project(0))).is_some());
    assert!(state.get(&key_of(&project(1))).is_none());
}

#[test]
fn the_single_entry_keys_of_older_versions_are_removed() {
    let old = "compact_key abc\ncompact_stamp 1:2;\ncompact_at 5\ncompact_window 300000\nz 1\n";
    let mut state = State::parse(old);
    render(&mut state, 0, 1000.0, &Cell::new(0));
    let keys = [
        "compact_key",
        "compact_stamp",
        "compact_at",
        "compact_window",
    ];
    assert!(keys.iter().all(|key| state.get(key).is_none()));
    assert_eq!(state.get("z"), Some("1"));
}

#[test]
fn a_malformed_entry_is_ignored_and_rewritten() {
    let key = key_of(&project(0));
    let bad = [
        "x",
        "1 2",
        "a b c",
        "5 1:2; 300000 more",
        "5 1:2; 0",
        "5 ; 300000",
    ];
    for value in bad {
        let mut state = State::parse(&format!("{} {}\n", key, value));
        let got = render(&mut state, 0, 1000.0, &Cell::new(0));
        assert_eq!(got, Some(200_000), "{}", value);
        assert_ne!(state.get(&key), Some(value));
        let again = render(&mut state, 0, 1001.0, &Cell::new(0));
        assert_eq!(again, Some(200_000), "{}", value);
    }
}

#[test]
fn malformed_entries_of_other_projects_are_dropped() {
    let mut state = State::parse("compact@zzz nonsense\n");
    render(&mut state, 0, 1000.0, &Cell::new(0));
    assert!(state.get("compact@zzz").is_none());
    assert_eq!(entries(&state), 1);
}

#[test]
fn an_evicted_entry_does_not_come_back_from_the_file() {
    let mut mine = State::parse("");
    let reads = Cell::new(0);
    for n in 0..SLOTS {
        render(&mut mine, n, 1000.0 + n as f64, &reads);
    }
    let on_disk = State::parse(&mine.text());
    let mut mine = State::parse(&mine.text());
    render(&mut mine, SLOTS, 1100.0, &reads);
    let saved = mine.rebased_on(on_disk);
    assert_eq!(entries(&saved), SLOTS);
    assert!(saved.get(&key_of(&project(0))).is_none());
}

#[test]
fn entries_of_a_project_and_its_model_are_kept_apart() {
    let mut state = State::parse("");
    let text = per_model(MODEL, "250000");
    let read = |f: &str| f.ends_with("file1").then(|| text.clone());
    let stat = |f: &str| f.ends_with("file1").then_some((10, 20));
    for model in [Some(MODEL), Some("claude-haiku-5")] {
        let given = inputs(model, Some(BIG));
        window(&given, &project(0), &mut state, 1000.0, &stat, &read);
    }
    assert_eq!(entries(&state), 2);
}
