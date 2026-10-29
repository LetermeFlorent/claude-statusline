use crate::backup;
use crate::check;
use crate::compact;
use crate::config::{Config, Metric};
use crate::global_config::{self, Repair};
use crate::messages::{self, Text};
use crate::os::{self, Claim, Contents};
use crate::plain;
use crate::portable;
use crate::render::{self, Data, SysValue};
use crate::session::Session;
use crate::settings::{self, Removal};
use crate::state::{self, State};
use crate::theme;
use crate::update;
use crate::update_watch;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

const TEMPLATE: &str = "{
  \"bar_width\": 8,
  \"bold\": true,
  \"glyphs\": { \"filled\": \"\u{25A0}\", \"empty\": \"\u{25A0}\", \"half\": \"\u{25AA}\" },
  \"auto_compact_marker\": \"~\",
  \"effort_style\": \"gradient\",
  \"segments\": { \"ctx\": true, \"5h\": true, \"7d\": true, \"model\": true },
  \"gradient\": { \"5h\": 0.55, \"7d\": 0.3 },
  \"system\": {
    \"enabled\": true,
    \"metrics\": [\"ram\", \"io\", \"cpu\"],
    \"gradient\": 0.45
  }
}
";

const MS_PER_SECOND: f64 = 1000.0;
const STATE_LOCK_WAIT_MS: u32 = 500;
const STATE_LOCK_SLICE_MS: u32 = 25;
const CHANGES_SUFFIX: &str = ".changes";
const STATE_READ_TRIES: u32 = 4;
const STATE_READ_RETRY_MS: u32 = 10;
const DEFAULT_BENCH_RUNS: usize = 200;

fn config_path() -> Option<String> {
    if let Some(p) = os::env("STATUSLINE_CONFIG") {
        return Some(p);
    }
    if let Some(d) = os::env("CLAUDE_CONFIG_DIR").filter(|d| !d.trim().is_empty()) {
        let p = os::join(&d, &["statusline.json"]);
        if os::exists(&p) {
            return Some(p);
        }
    }
    os::home().map(|h| os::join(&h, &[".claude", "statusline.json"]))
}

fn state_path() -> Option<String> {
    os::state_dir().map(|d| os::join(&d, &["state"]))
}

fn read_state(path: &str) -> Option<State> {
    for attempt in 0..STATE_READ_TRIES {
        if attempt > 0 {
            os::sleep(STATE_READ_RETRY_MS);
        }
        match os::read_checked(path, os::MAX_READ_BYTES) {
            Contents::Bytes(data) => return Some(State::parse(&String::from_utf8_lossy(&data))),
            Contents::Missing => return Some(State::parse("")),
            Contents::Unreadable => {}
        }
    }
    None
}

fn pending_changes(path: &str) -> Vec<String> {
    let Some(dir) = os::parent(path) else {
        return Vec::new();
    };
    let prefix = format!("{}.", os::file_name(path));
    let mut found: Vec<String> = os::list(dir, &prefix)
        .into_iter()
        .filter(|name| name.ends_with(CHANGES_SUFFIX))
        .map(|name| os::join(dir, &[&name]))
        .collect();
    found.sort();
    found
}

fn read_pending(path: &str) -> Vec<(String, String)> {
    pending_changes(path)
        .into_iter()
        .filter_map(|file| os::read_text(&file).map(|changes| (file, changes)))
        .collect()
}

fn with_pending(mut state: State, pending: &[(String, String)]) -> State {
    for (_, changes) in pending {
        state.apply(changes);
    }
    state
}

fn load_state() -> State {
    let Some(path) = state_path() else {
        return State::parse("");
    };
    let pending = read_pending(&path);
    let state = read_state(&path).unwrap_or_else(|| State::parse(""));
    if pending.is_empty() {
        return state;
    }
    State::parse(&with_pending(state, &pending).text())
}

fn merge_state(path: &str, own: Option<&State>) -> bool {
    let pending = read_pending(path);
    let Some(latest) = read_state(path) else {
        return own.is_some_and(|state| os::write_atomic(path, state.text().as_bytes()));
    };
    let mut merged = with_pending(latest, &pending);
    if let Some(state) = own {
        merged.apply(&state.changes());
    }
    let written = merged.changed() && os::write_atomic(path, merged.text().as_bytes());
    if written || !merged.changed() {
        for (file, _) in &pending {
            os::remove(file);
        }
    }
    written
}

fn park_changes(path: &str, lock_path: &str, state: &State) -> bool {
    let parked = format!("{}.{}{}", path, os::pid(), CHANGES_SUFFIX);
    if !os::write_atomic(&parked, state.changes().as_bytes()) {
        return false;
    }
    for _ in 0..STATE_LOCK_WAIT_MS / STATE_LOCK_SLICE_MS {
        if !os::exists(&parked) {
            break;
        }
        if let Claim::Held(_lock) = os::lock(lock_path, STATE_LOCK_SLICE_MS) {
            merge_state(path, None);
            break;
        }
    }
    true
}

fn save_state(state: &State) -> bool {
    state_path().is_some_and(|path| save_state_at(&path, state))
}

fn save_state_at(path: &str, state: &State) -> bool {
    if !state.changed() {
        return false;
    }
    if let Some(d) = os::parent(path) {
        if !os::exists(d) {
            os::mkdirs(d);
        }
    }
    let lock_path = format!("{}.lock", path);
    let claim = os::lock(&lock_path, 0);
    if let Claim::Busy = claim {
        return park_changes(path, &lock_path, state);
    }
    merge_state(path, Some(state))
}

fn load_config() -> Config {
    Config::parse(config_path().and_then(|p| os::read_text(&p)).as_deref())
}

#[derive(Clone, Copy)]
enum Stage {
    State,
    Config,
    Session,
    Theme,
    Compact,
    System,
    Render,
    Write,
}

const STAGE_COUNT: usize = Stage::Write as usize + 1;
const BENCH_THEME: Text = Text::new("theme", "thème");
const BENCH_COMPACT: Text = Text::new("compact", "compact");
const BENCH_SYSTEM: Text = Text::new("system", "système");
const STAGE_LABELS: [(Stage, &Text); STAGE_COUNT] = [
    (Stage::State, &messages::BENCH_STATE),
    (Stage::Config, &messages::BENCH_CONFIG),
    (Stage::Session, &messages::BENCH_SESSION),
    (Stage::Theme, &BENCH_THEME),
    (Stage::Compact, &BENCH_COMPACT),
    (Stage::System, &BENCH_SYSTEM),
    (Stage::Render, &messages::BENCH_RENDER),
    (Stage::Write, &messages::BENCH_WRITE),
];

struct Laps {
    seconds: [f64; STAGE_COUNT],
    last: f64,
}

impl Laps {
    fn new() -> Laps {
        Laps {
            seconds: [0.0; STAGE_COUNT],
            last: os::ticks(),
        }
    }
    fn lap(&mut self, stage: Stage) {
        let now = os::ticks();
        self.seconds[stage as usize] += now - self.last;
        self.last = now;
    }
    fn spent(&self, stage: Stage) -> f64 {
        self.seconds[stage as usize]
    }
}

fn sample(cfg: &Config, state: &mut State, now: f64) -> Vec<SysValue> {
    let mut out = Vec::new();
    if !cfg.sys_enabled {
        return out;
    }
    for metric in &cfg.metrics {
        let value = match metric {
            Metric::Ram => SysValue {
                metric: *metric,
                pct: None,
                ram: os::memory(),
            },
            Metric::Io => SysValue {
                metric: *metric,
                pct: state.counter("io", os::disk(&cfg.disk_path), now),
                ram: None,
            },
            Metric::Cpu => SysValue {
                metric: *metric,
                pct: state.counter("cpu", os::cpu(), now),
                ram: None,
            },
        };
        out.push(value);
    }
    out
}

fn local_only(dir: Option<&str>) -> Option<&str> {
    dir.filter(|dir| !os::on_network(dir))
}

fn compact_files(cwd: Option<&str>) -> [String; 3] {
    let main = os::claude_dir()
        .map(|d| os::join(&d, &["settings.json"]))
        .unwrap_or_default();
    let cwd = cwd.map(String::from).or_else(os::cwd);
    let folder = local_only(cwd.as_deref());
    let project = |f: &str| {
        folder
            .map(|c| os::join(c, &[".claude", f]))
            .unwrap_or_default()
    };
    [
        main,
        project("settings.json"),
        project("settings.local.json"),
    ]
}

fn stat_file(p: &str) -> Option<(u64, i64)> {
    if p.is_empty() {
        None
    } else {
        os::stamp(p)
    }
}

fn read_file(p: &str) -> Option<String> {
    if p.is_empty() {
        None
    } else {
        os::read_text(p)
    }
}

fn run(input: &str, now: f64, laps: &mut Laps) -> (String, State, bool) {
    let state = load_state();
    laps.lap(Stage::State);
    run_on(input, state, now, laps)
}

fn run_on(input: &str, mut state: State, now: f64, laps: &mut Laps) -> (String, State, bool) {
    let cfg = load_config();
    laps.lap(Stage::Config);
    let session = Session::parse(input);
    let quotas = state.quotas(
        session.live,
        now,
        &state::account_tag(&os::claude_dir().unwrap_or_default()),
    );
    laps.lap(Stage::Session);
    let project = local_only(session.project.as_deref().or(session.cwd.as_deref()));
    let light = theme::detect_light(cfg.light_override, project, &mut state);
    laps.lap(Stage::Theme);
    let window = if cfg.seg_ctx {
        let env = os::env("CLAUDE_CODE_AUTO_COMPACT_WINDOW");
        let files = compact_files(session.cwd.as_deref());
        compact::window(
            env.as_deref(),
            &files,
            &mut state,
            now,
            &stat_file,
            &read_file,
        )
    } else {
        None
    };
    laps.lap(Stage::Compact);
    let sys = sample(&cfg, &mut state, now);
    laps.lap(Stage::System);
    let updating = update_watch::busy(now);
    let data = Data {
        ctx: session.ctx,
        window,
        quotas,
        model: session.model,
        effort: session.effort,
        sys,
        updating,
    };
    let out = render::render(&cfg, &cfg.colors(light), &data, now);
    let out = if plain::wanted(&os::env) {
        plain::strip(&out)
    } else {
        out
    };
    laps.lap(Stage::Render);
    (out, state, cfg.auto_update)
}

fn demo_input(now: f64) -> String {
    let now = now as i64;
    format!(
        "{{\"model\":{{\"id\":\"claude-opus-5\",\"display_name\":\"Opus 5\"}},\"effort\":{{\"level\":\"max\"}},\
\"context_window\":{{\"context_window_size\":1000000,\"used_percentage\":34,\
\"current_usage\":{{\"input_tokens\":12000,\"cache_creation_input_tokens\":48000,\"cache_read_input_tokens\":280000}}}},\
\"rate_limits\":{{\"five_hour\":{{\"used_percentage\":36,\"resets_at\":{}}},\"seven_day\":{{\"used_percentage\":61,\"resets_at\":{}}}}}}}",
        now + 12600,
        now + 320400
    )
}

struct Rounds {
    laps: Laps,
    bytes: usize,
    seconds: f64,
}

fn round(input: &str, target: Option<&str>, laps: &mut Laps) -> usize {
    laps.last = os::ticks();
    let state = target
        .and_then(read_state)
        .unwrap_or_else(|| State::parse(""));
    laps.lap(Stage::State);
    let (out, state, _) = run_on(input, state, os::now(), laps);
    if let Some(path) = target {
        save_state_at(path, &state);
    }
    laps.lap(Stage::Write);
    out.len()
}

fn rounds(input: &str, target: Option<&str>, runs: usize) -> Rounds {
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

fn bench_target(state: &str, pid: u32) -> String {
    format!("{}-bench-{}", state, pid)
}

fn clear_bench(target: &str) {
    for file in pending_changes(target) {
        os::remove(&file);
    }
    os::remove(&format!("{}.lock", target));
    os::remove(target);
}

fn bench(runs: usize) {
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

fn bench_runs(count: Option<&str>) -> Option<usize> {
    match count {
        None => Some(DEFAULT_BENCH_RUNS),
        Some(text) => text.parse::<usize>().ok().filter(|runs| *runs > 0),
    }
}

fn settings_path() -> Option<String> {
    os::claude_dir().map(|d| os::join(&d, &["settings.json"]))
}

fn fail(msg: &str) -> i32 {
    os::err(msg);
    1
}

fn store(path: &str, text: &str) -> Result<(), String> {
    if os::write_atomic(path, text.as_bytes()) {
        Ok(())
    } else {
        Err(messages::CANNOT_WRITE.with(&[path]))
    }
}

fn replace(path: &str, raw: &[u8], text: &str) -> Result<String, String> {
    let saved =
        backup::save(path, raw, os::now()).map_err(|name| messages::CANNOT_WRITE.with(&[&name]))?;
    let told = messages::BACKUP.with(&[&saved]);
    match store(path, text) {
        Ok(()) => Ok(told),
        Err(message) => Err(told + &message),
    }
}

fn repair_global(path: &str, command: &str) -> Result<Option<String>, ()> {
    let raw = match os::read_checked(path, global_config::MAX_GLOBAL_BYTES) {
        Contents::Missing => return Ok(None),
        Contents::Unreadable => return Err(()),
        Contents::Bytes(raw) => raw,
    };
    let text = core::str::from_utf8(&raw).map_err(|_| ())?;
    match global_config::repair(text, command, global_config::present) {
        Repair::Kept => Ok(None),
        Repair::Unreadable => Err(()),
        Repair::Repaired { text: next, was } => match replace(path, &raw, &next) {
            Ok(told) => {
                os::out(&told);
                Ok(Some(was))
            }
            Err(message) => {
                os::err(&message);
                Err(())
            }
        },
    }
}

fn repair_globals(command: &str) {
    let Some(dir) = os::claude_dir() else { return };
    for path in global_config::candidates(&dir, os::home().as_deref()) {
        match repair_global(&path, command) {
            Ok(Some(was)) => os::out(&messages::GLOBAL_REPAIRED.with(&[&path, &was])),
            Ok(None) => {}
            Err(()) => os::err(&messages::GLOBAL_KEPT.with(&[&path])),
        }
    }
}

fn settings_bytes(path: &str) -> Result<Option<Vec<u8>>, String> {
    match os::read_checked(path, os::MAX_READ_BYTES) {
        Contents::Missing => Ok(None),
        Contents::Bytes(raw) => Ok(Some(raw)),
        Contents::Unreadable => Err(messages::SETTINGS_UNREADABLE.with(&[path])),
    }
}

fn add_line(path: &str, command: &str) -> Result<String, String> {
    let raw = settings_bytes(path)?;
    let next = settings::with_status_line(raw.as_deref(), command)
        .ok_or_else(|| messages::SETTINGS_INVALID.with(&[path]))?;
    let told = match raw {
        Some(raw) => replace(path, &raw, &next)?,
        None => {
            if let Some(d) = os::parent(path) {
                os::mkdirs(d);
            }
            store(path, &next)?;
            String::new()
        }
    };
    Ok(told + &messages::INSTALLED.with(&[path]))
}

fn remove_line(path: &str) -> Result<String, String> {
    let Some(raw) = settings_bytes(path)? else {
        return Ok(messages::NO_SETTINGS.with(&[path]));
    };
    let next = match settings::without_status_line(&raw) {
        Removal::Invalid => return Err(messages::SETTINGS_INVALID.with(&[path])),
        Removal::NoLine => return Ok(messages::NOTHING_TO_REMOVE.with(&[path])),
        Removal::Rewrite(next) => next,
    };
    let told = replace(path, &raw, &next)?;
    Ok(told + &messages::REMOVED.with(&[path]))
}

fn install() -> i32 {
    let Some(path) = settings_path() else {
        return fail(messages::NO_HOME.text());
    };
    let Some(exe) = os::exe() else {
        return fail(messages::NO_EXE.text());
    };
    let command = portable::command(&exe);
    match add_line(&path, &command) {
        Ok(told) => os::out(&told),
        Err(message) => return fail(&message),
    }
    repair_globals(&command);
    let conf = config_path();
    if let Some(c) = conf.filter(|c| !os::exists(c)) {
        let target = os::claude_dir()
            .map(|d| os::join(&d, &["statusline.json"]))
            .unwrap_or(c);
        if os::write_atomic(&target, TEMPLATE.as_bytes()) {
            os::out(&messages::CONFIG_WRITTEN.with(&[&target]));
        }
    }
    os::out(messages::RESTART.text());
    0
}

fn uninstall() -> i32 {
    let Some(path) = settings_path() else {
        return fail(messages::NO_HOME.text());
    };
    match remove_line(&path) {
        Ok(told) => {
            os::out(&told);
            0
        }
        Err(message) => fail(&message),
    }
}

pub fn main(args: Vec<String>) -> i32 {
    match args.get(1).map(|s| s.as_str()) {
        None => {
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
        Some("--demo") => {
            let now = os::now();
            let (out, _, _) = run(&demo_input(now), now, &mut Laps::new());
            os::out(&out);
            0
        }
        Some("--bench") => {
            let count = args.get(2).map(String::as_str);
            let Some(runs) = bench_runs(count) else {
                os::err(&messages::BAD_BENCH_COUNT.with(&[count.unwrap_or_default()]));
                return 2;
            };
            bench(runs);
            0
        }
        Some("--theme") => {
            let cfg = load_config();
            let mut state = load_state();
            let cwd = os::cwd();
            let decision =
                theme::detect(cfg.light_override, local_only(cwd.as_deref()), &mut state);
            save_state(&state);
            os::out(&decision.report());
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
            os::err(&messages::UNKNOWN_ARGUMENT.with(&[other, messages::HELP.text()]));
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::Sandbox;

    #[test]
    fn uninstall_names_each_case() {
        let sandbox = Sandbox::new("uninstall-cases");
        let path = sandbox.path("settings.json");
        assert_eq!(remove_line(&path), Ok(messages::NO_SETTINGS.with(&[&path])));
        for text in ["", "{\"model\": \"opus\"}"] {
            std::fs::write(&path, text).unwrap();
            let no_line = Ok(messages::NOTHING_TO_REMOVE.with(&[&path]));
            assert_eq!(remove_line(&path), no_line, "{}", text);
        }
        for raw in [&b"[1]"[..], b"{\"a\": \"\xff\"}"] {
            std::fs::write(&path, raw).unwrap();
            let invalid = Err(messages::SETTINGS_INVALID.with(&[&path]));
            assert_eq!(remove_line(&path), invalid);
            assert_eq!(add_line(&path, "x"), invalid);
            assert_eq!(std::fs::read(&path).unwrap(), raw);
        }
        assert_eq!(sandbox.files(), ["settings.json"]);
    }

    #[test]
    fn unreadable_settings_are_an_error() {
        let sandbox = Sandbox::new("unreadable-settings");
        let path = sandbox.path("settings.json");
        std::fs::create_dir(&path).unwrap();
        let unreadable = Err(messages::SETTINGS_UNREADABLE.with(&[&path]));
        assert_eq!(remove_line(&path), unreadable);
        assert_eq!(add_line(&path, "x"), unreadable);
    }

    #[test]
    fn an_empty_file_counts_as_an_empty_object() {
        let sandbox = Sandbox::new("empty-settings");
        let path = sandbox.path("settings.json");
        std::fs::write(&path, " \n").unwrap();
        assert!(add_line(&path, "x").is_ok());
        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("\"statusLine\""), "{}", written);
    }

    #[test]
    fn the_printed_backup_is_on_disk() {
        let sandbox = Sandbox::new("printed-backup");
        let path = sandbox.path("settings.json");
        std::fs::write(&path, "{\"statusLine\": {}}").unwrap();
        let told = remove_line(&path).unwrap();
        let files = sandbox.files();
        let backup = files.iter().find(|name| *name != "settings.json").unwrap();
        assert!(backup.starts_with("settings.json.bak-"), "{}", backup);
        let backup_told = messages::BACKUP.with(&[&sandbox.path(backup)]);
        assert_eq!(told, backup_told + &messages::REMOVED.with(&[&path]));
    }

    #[cfg(windows)]
    #[test]
    fn a_state_file_held_a_moment_without_read_sharing_is_still_read() {
        use std::os::windows::fs::OpenOptionsExt;
        const SHARE_WRITE_AND_DELETE: u32 = 6;
        const HELD_MS: u64 = 5;
        let sandbox = Sandbox::new("held-state");
        let path = sandbox.path("state");
        std::fs::write(&path, "q5_pct@a 40\n").unwrap();
        let holder = std::fs::OpenOptions::new()
            .write(true)
            .share_mode(SHARE_WRITE_AND_DELETE)
            .open(&path)
            .unwrap();
        let release = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(HELD_MS));
            drop(holder);
        });
        let state = read_state(&path);
        release.join().unwrap();
        let pct = state.as_ref().and_then(|s| s.get("q5_pct@a"));
        assert_eq!(pct, Some("40"));
    }

    #[test]
    fn the_written_template_passes_the_check() {
        assert!(check::problems(TEMPLATE).is_empty());
    }

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
}
