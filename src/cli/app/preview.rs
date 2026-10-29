use super::persist::load_state;
use super::run::{run_on, Laps};
use super::USAGE_EXIT;
use crate::config::Config;
use crate::messages;
use crate::os;
use crate::render::demo::{self, Level};
use crate::state::State;
use alloc::format;
use alloc::string::String;

const DEMO_TOKENS_PER_PERCENT: u64 = 10_000;
const DEMO_FRESH_TOKENS: u64 = 60_000;

pub(super) fn demo_input(now: f64) -> String {
    demo_input_for(now, None)
}

fn demo_input_for(now: f64, level: Option<Level>) -> String {
    let now = now as i64;
    let percent = level.unwrap_or(Level::Low).context_percent();
    let cached = percent * DEMO_TOKENS_PER_PERCENT - DEMO_FRESH_TOKENS;
    format!(
        "{{\"model\":{{\"id\":\"claude-opus-5\",\"display_name\":\"Opus 5\"}},\"effort\":{{\"level\":\"max\"}},\
\"context_window\":{{\"context_window_size\":1000000,\"used_percentage\":{},\
\"current_usage\":{{\"input_tokens\":12000,\"cache_creation_input_tokens\":48000,\"cache_read_input_tokens\":{}}}}},\
\"rate_limits\":{{\"five_hour\":{{\"used_percentage\":36,\"resets_at\":{}}},\"seven_day\":{{\"used_percentage\":61,\"resets_at\":{}}}}}}}",
        percent,
        cached,
        now + 12600,
        now + 320400
    )
}

pub(super) fn show_demo(word: Option<&str>) -> i32 {
    let level = match word.map(|word| (word, Level::named(word))) {
        None => None,
        Some((_, Some(level))) => Some(level),
        Some((word, None)) => {
            let message = messages::UNKNOWN_ARGUMENT.with(&[word, messages::HELP.text()]);
            return os::fail(&message, USAGE_EXIT);
        }
    };
    let now = os::now();
    let readings = |cfg: &Config, _: &mut State, _: f64| demo::readings(cfg, level, os::memory());
    let (out, _, _) = run_on(
        &demo_input_for(now, level),
        load_state(),
        now,
        &mut Laps::new(),
        &readings,
    );
    os::out(&out);
    0
}
