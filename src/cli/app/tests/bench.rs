use crate::app::bench::*;
use crate::app::persist::{pending_changes, CHANGES_SUFFIX};
use crate::app::preview::demo_input;
use crate::app::run::{Laps, STAGE_LABELS};
use crate::os;
use crate::sandbox::Sandbox;

#[test]
fn a_bench_round_carries_the_state_of_the_previous_round() {
    let sandbox = Sandbox::new("bench-round");
    let target = sandbox.path("state.bench");
    std::fs::write(&target, "kept 1\n").unwrap();
    let input = demo_input(os::now());
    round(&input, Some(&target), &mut Laps::new());
    let written = std::fs::read_to_string(&target).unwrap();
    assert!(written.contains("kept 1\n"), "{}", written);
    assert_eq!(sandbox.files(), ["state.bench", "state.bench.lock"]);
}

#[test]
fn the_bench_leaves_the_state_folder_as_it_found_it() {
    let sandbox = Sandbox::new("bench-clean");
    std::fs::write(
        sandbox.path("state"),
        "kept 1
",
    )
    .unwrap();
    let before = sandbox.files();
    let target = bench_target(&sandbox.path("state"), 7);
    rounds(&demo_input(os::now()), Some(&target), 2);
    let parked = format!("{}.9{}", target, CHANGES_SUFFIX);
    std::fs::write(
        &parked, "kept 2
",
    )
    .unwrap();
    assert_ne!(sandbox.files(), before);
    clear_bench(&target);
    assert_eq!(sandbox.files(), before);
}

#[test]
fn two_bench_targets_of_different_pids_do_not_touch_each_other() {
    let sandbox = Sandbox::new("bench-pids");
    let state = sandbox.path("state");
    let (first, second) = (bench_target(&state, 1), bench_target(&state, 2));
    assert_ne!(first, second);
    let input = demo_input(os::now());
    rounds(&input, Some(&first), 1);
    let kept = std::fs::read_to_string(&first).unwrap();
    rounds(&input, Some(&second), 2);
    clear_bench(&second);
    assert_eq!(std::fs::read_to_string(&first).unwrap(), kept);
    assert_eq!(sandbox.files(), ["state-bench-1", "state-bench-1.lock"]);
}

#[test]
fn a_bench_target_is_never_a_changes_file_of_the_real_state() {
    let sandbox = Sandbox::new("bench-name");
    let state = sandbox.path("state");
    let target = bench_target(&state, 3);
    std::fs::write(
        format!("{}.4{}", target, CHANGES_SUFFIX),
        "x 1
",
    )
    .unwrap();
    assert!(pending_changes(&state).is_empty());
}

#[test]
fn the_bench_starts_from_an_empty_state() {
    let sandbox = Sandbox::new("bench-empty");
    let target = sandbox.path("state.bench");
    std::fs::write(&target, "left 1\n").unwrap();
    rounds(&demo_input(os::now()), Some(&target), 1);
    let written = std::fs::read_to_string(&target).unwrap();
    assert!(!written.contains("left"), "{}", written);
}

#[test]
fn each_bench_stage_has_its_own_label() {
    for (at, (stage, _)) in STAGE_LABELS.into_iter().enumerate() {
        assert_eq!(stage as usize, at);
    }
}

#[test]
fn bench_without_a_count_uses_the_default() {
    assert_eq!(bench_runs(None), Some(DEFAULT_BENCH_RUNS));
}

#[test]
fn bench_takes_a_whole_number_above_zero() {
    assert_eq!(bench_runs(Some("5")), Some(5));
    for count in ["abc", "0", "-3", "1.5", "", "1e3"] {
        assert_eq!(bench_runs(Some(count)), None, "{}", count);
    }
}
