use crate::messages::{self, Text};
use crate::os;

#[derive(Clone, Copy)]
pub(in crate::cli::app) enum Stage {
    State,
    Config,
    Session,
    Theme,
    Compact,
    System,
    Render,
    Write,
}

pub(in crate::cli::app) const STAGE_COUNT: usize = Stage::Write as usize + 1;
pub(in crate::cli::app) const STAGE_LABELS: [(Stage, &Text); STAGE_COUNT] = [
    (Stage::State, &messages::BENCH_STATE),
    (Stage::Config, &messages::BENCH_CONFIG),
    (Stage::Session, &messages::BENCH_SESSION),
    (Stage::Theme, &messages::BENCH_THEME),
    (Stage::Compact, &messages::BENCH_COMPACT),
    (Stage::System, &messages::BENCH_SYSTEM),
    (Stage::Render, &messages::BENCH_RENDER),
    (Stage::Write, &messages::BENCH_WRITE),
];

pub(in crate::cli::app) struct Laps {
    seconds: [f64; STAGE_COUNT],
    pub(in crate::cli::app) last: f64,
}

impl Laps {
    pub(in crate::cli::app) fn new() -> Laps {
        Laps {
            seconds: [0.0; STAGE_COUNT],
            last: os::ticks(),
        }
    }
    pub(in crate::cli::app) fn lap(&mut self, stage: Stage) {
        let now = os::ticks();
        self.seconds[stage as usize] += now - self.last;
        self.last = now;
    }
    pub(in crate::cli::app) fn spent(&self, stage: Stage) -> f64 {
        self.seconds[stage as usize]
    }
}
