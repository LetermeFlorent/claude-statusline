use crate::render::text::{pct_text, reset_text, tokens};

#[test]
fn labels_shorten_and_cells_stay_within_the_bar() {
    assert_eq!(tokens(2500), "3k");
    assert_eq!(tokens(3500), "4k");
    assert_eq!(tokens(999), "999");
    assert_eq!(reset_text(59.9, false), "0m");
    assert_eq!(reset_text(5000.0, false), "1h23m");
    assert_eq!(reset_text(200000.0, true), "2d07h");
    assert_eq!(reset_text(-5.0, true), "0m");
}

#[test]
fn huge_values_are_clamped() {
    assert_eq!(tokens(u64::MAX), "999B");
    assert_eq!(reset_text(1e300, true), "99d00h");
    assert_eq!(reset_text(f64::INFINITY, false), "2376h00m");
    assert_eq!(reset_text(f64::NAN, true), "0m");
}

#[test]
fn a_count_close_to_a_million_moves_to_the_next_unit() {
    assert_eq!(tokens(999_499), "999k");
    assert_eq!(tokens(999_500), "1M");
    assert_eq!(tokens(999_999), "1M");
    assert_eq!(tokens(999_499_999), "999M");
    assert_eq!(tokens(999_500_000), "1B");
    assert_eq!(tokens(1_499), "1k");
    assert_eq!(tokens(1_500), "2k");
    assert_eq!(tokens(2_500), "3k");
}

#[test]
fn negative_percent_reads_zero() {
    assert_eq!(pct_text(Some(-3.0)), "0%");
}
