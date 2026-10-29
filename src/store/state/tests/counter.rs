use crate::os::Activity;
use crate::state::State;

fn reading(source: u64, clock: i64, busy: i64) -> Option<Activity> {
    Some(Activity {
        source,
        clock,
        busy,
    })
}

#[test]
fn counter_percent_comes_from_the_previous_reading() {
    let mut state = State::parse("cpu_source 0\ncpu_clock 100\ncpu_value 10\ncpu_at 1000\n");
    assert_eq!(
        state.counter("cpu", reading(0, 200, 60), 1003.0),
        Some(50.0)
    );
    assert_eq!(state.counter("cpu", reading(0, 200, 70), 1006.0), None);
    assert_eq!(state.counter("io", None, 1006.0), None);
    assert_eq!(state.get("cpu_value"), Some("70"));
}

#[test]
fn a_reading_from_another_source_starts_over() {
    let mut state = State::parse("");
    assert_eq!(state.counter("io", reading(1, 100, 10), 1000.0), None);
    assert_eq!(state.counter("io", reading(2, 200, 90), 1003.0), None);
    assert_eq!(
        state.counter("io", reading(2, 300, 140), 1006.0),
        Some(50.0)
    );
}

#[test]
fn close_readings_keep_the_reference_and_late_ones_give_no_percent() {
    let mut state = State::parse("");
    assert_eq!(state.counter("cpu", reading(0, 100, 10), 1000.0), None);
    assert_eq!(
        state.counter("cpu", reading(0, 200, 60), 1003.0),
        Some(50.0)
    );
    assert_eq!(
        state.counter("cpu", reading(0, 201, 61), 1003.01),
        Some(50.0)
    );
    assert_eq!(state.get("cpu_clock"), Some("200"));
    assert_eq!(state.counter("cpu", reading(0, 300, 160), 1123.0), None);
    assert_eq!(
        state.counter("cpu", reading(0, 400, 210), 1126.0),
        Some(50.0)
    );
}
