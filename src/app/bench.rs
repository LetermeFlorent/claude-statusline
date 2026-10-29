use super::paths::state_path;
use super::persist::{pending_changes, read_state, save_state_at};
use super::preview::demo_input;
use super::run::{run_on, sample, Laps, Stage, STAGE_LABELS};
use crate::messages;
use crate::os;
use crate::state::State;
use alloc::format;
use alloc::string::{String, ToString};

const MS_PER_SECOND: f64 = 1000.0;
pub(super) const DEFAULT_BENCH_RUNS: usize = 200;

pub(super) struct Rounds {
    laps: Laps,
    bytes: usize,
    seconds: f64,
}

pub(super) fn round(input: &str, target: Option<&str>, laps: &mut Laps) -> usize {
    laps.last = os::ticks();
    let state = target
        .and_then(read_state)
        .unwrap_or_else(|| State::parse(""));
    laps.lap(Stage::State);
    let (out, state, _) = run_on(input, state, os::now(), laps, &sample);
    if let Some(path) = target {
        save_state_at(path, &state);
    }
    laps.lap(Stage::Write);
    out.len()
}

pub(super) fn rounds(input: &str, target: Option<&str>, runs: usize) -> Rounds {
    if let Some(path) = target {
        clear_bench(path);
    }
    let mut laps = Laps::new();
    let mut bytes = 0;
    let start = os::ticks();
    for _ in 0..runs {
        bytes = round(input, target, &mut laps);
    }
    Rounds {
        laps,
        bytes,
        seconds: os::ticks() - start,
    }
}

pub(super) fn bench_target(state: &str, pid: u32) -> String {
    format!("{}-bench-{}", state, pid)
}

pub(super) fn clear_bench(target: &str) {
    for file in pending_changes(target) {
        os::remove(&file);
    }
    os::remove(&format!("{}.lock", target));
    os::remove(target);
}

pub(super) fn bench(runs: usize) {
    let input = demo_input(os::now());
    let target = state_path().map(|p| bench_target(&p, os::pid()));
    if let Some(d) = target.as_deref().and_then(os::parent) {
        if !os::exists(d) {
            os::mkdirs(d);
        }
    }
    let done = rounds(&input, target.as_deref(), runs);
    if let Some(path) = &target {
        clear_bench(path);
    }
    let per_run_ms = |seconds: f64| format!("{:.3}", seconds * MS_PER_SECOND / runs as f64);
    let mut report = messages::BENCH_SUMMARY.with(&[
        &runs.to_string(),
        &per_run_ms(done.seconds),
        &done.bytes.to_string(),
    ]);
    for (stage, label) in STAGE_LABELS {
        report.push_str(&format!(
            "  {:<10}{} ms\n",
            label.text(),
            per_run_ms(done.laps.spent(stage))
        ));
    }
    os::out(&report);
}

pub(super) fn bench_runs(count: Option<&str>) -> Option<usize> {
    match count {
        None => Some(DEFAULT_BENCH_RUNS),
        Some(text) => text.parse::<usize>().ok().filter(|runs| *runs > 0),
    }
}
