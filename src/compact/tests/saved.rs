use super::cache::{entries, key_of, project, render, SLOTS};
use super::*;
use core::cell::Cell;

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
    let mut saved = on_disk;
    saved.apply(&mine.changes());
    assert_eq!(entries(&saved), SLOTS);
    assert!(saved.get(&key_of(&project(0))).is_none());
}

#[test]
fn old_keys_do_not_come_back_from_the_file() {
    let old = "compact_key abc
compact_stamp 1:2;
compact_at 5
compact_window 300000
";
    let mut mine = State::parse(old);
    render(&mut mine, 0, 1000.0, &Cell::new(0));
    let mut saved = State::parse(old);
    saved.apply(&mine.changes());
    assert_eq!(entries(&saved), 1);
    assert_eq!(saved.with_prefix("compact_").count(), 0);
}
