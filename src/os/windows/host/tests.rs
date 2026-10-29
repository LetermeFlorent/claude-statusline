use super::console::in_utf16_chunks;
use crate::os::windows::sys::DiskPerformance;
use crate::state::State;
use alloc::vec::Vec;

#[test]
fn console_text_is_cut_between_characters() {
    const HIGH_SURROGATES: core::ops::RangeInclusive<u16> = 0xD800..=0xDBFF;
    let text = "a\u{10348}bc\u{e9}\u{10348}\u{10348}d";
    let mut buffer = [0u16; 3];
    let mut chunks: Vec<Vec<u16>> = Vec::new();
    assert!(in_utf16_chunks(text, &mut buffer, &mut |chunk| {
        chunks.push(chunk.to_vec());
        true
    }));
    assert_eq!(chunks.concat(), text.encode_utf16().collect::<Vec<u16>>());
    for chunk in &chunks {
        assert!(!chunk.is_empty() && chunk.len() <= buffer.len());
        assert!(!HIGH_SURROGATES.contains(chunk.last().unwrap()));
    }
}

#[test]
fn a_failed_console_write_stops_the_text() {
    let mut calls = 0;
    let written = in_utf16_chunks("abcdef", &mut [0u16; 2], &mut |_| {
        calls += 1;
        false
    });
    assert!(!written);
    assert_eq!(calls, 1);
}

#[test]
fn overlapping_requests_count_the_busy_time_once() {
    let before = DiskPerformance {
        read_time: 40_000_000,
        idle_time: 70_000_000,
        query_time: 100_000_000,
        ..Default::default()
    };
    let after = DiskPerformance {
        read_time: 70_000_000,
        idle_time: 75_000_000,
        query_time: 110_000_000,
        ..Default::default()
    };
    let mut state = State::parse("");
    assert_eq!(state.counter("io", Some(before.activity(1)), 1000.0), None);
    assert_eq!(
        state.counter("io", Some(after.activity(1)), 1005.0),
        Some(50.0)
    );
}
