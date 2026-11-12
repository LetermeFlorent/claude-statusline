mod bench;
mod install;
mod paths;
mod persist;
mod preview;
mod run;
mod subagents;
#[cfg(test)]
mod tests;

use crate::check;
use crate::messages;
use crate::os;
use crate::theme;
use crate::update;
use crate::update_watch;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use bench::{bench, bench_runs};
use install::{install, uninstall};
use paths::{config_path, load_config};
use persist::{load_state, save_state};
use preview::show_demo;
use run::{local_only, run, Laps};
use subagents::subagents;

const FAILED_EXIT: i32 = 1;
const USAGE_EXIT: i32 = 2;
const VALUE_ARGUMENT: usize = 2;

fn status_line() -> i32 {
    let input = os::stdin().unwrap_or_default();
    let now = os::now();
    let (out, mut state, auto_update) = run(&input, now, &mut Laps::new());
    os::out(&out);
    let due = auto_update && update_watch::due(&mut state, now);
    if save_state(&state) && due {
        update_watch::launch();
    }
    0
}

fn theme_report() -> i32 {
    let cfg = load_config();
    let mut state = load_state();
    let cwd = os::cwd();
    let project = local_only(cwd.as_deref());
    let decision = theme::detect(cfg.light_override, project, &mut state, os::now());
    save_state(&state);
    os::out(&decision.report());
    0
}

pub fn main(args: Vec<String>) -> i32 {
    match args.get(1).map(|s| s.as_str()) {
        None => status_line(),
        Some("subagents") => subagents(),
        Some("--demo") => show_demo(args.get(VALUE_ARGUMENT).map(String::as_str)),
        Some("--bench") => {
            let count = args.get(VALUE_ARGUMENT).map(String::as_str);
            let Some(runs) = bench_runs(count) else {
                let message = messages::BAD_BENCH_COUNT.with(&[count.unwrap_or_default()]);
                return os::fail(&message, USAGE_EXIT);
            };
            bench(runs);
            0
        }
        Some("--theme") => theme_report(),
        Some("--language") => {
            os::out(&format!("{}\n", messages::language_code()));
            0
        }
        Some("--check") => check::run(config_path()),
        Some("--install") => install(),
        Some("--uninstall") => uninstall(),
        Some("--update") => update::run(),
        Some("--version") | Some("-V") => {
            os::out(concat!("statusline ", env!("CARGO_PKG_VERSION"), "\n"));
            0
        }
        Some("--help") | Some("-h") => {
            os::out(messages::HELP.text());
            0
        }
        Some(other) => {
            let message = messages::UNKNOWN_ARGUMENT.with(&[other, messages::HELP.text()]);
            os::fail(&message, USAGE_EXIT)
        }
    }
}
