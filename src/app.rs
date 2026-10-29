use crate::backup;
use crate::check;
use crate::compact;
use crate::config::{Config, Metric};
use crate::global_config::{self, Repair};
use crate::messages;
use crate::os::{self, Contents};
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
  \"auto_compact_marker\": \"\u{AA}\",
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

fn load_state() -> State {
    State::parse(
        &state_path()
            .and_then(|p| os::read_text(&p))
            .unwrap_or_default(),
    )
}

fn save_state(state: &State) -> bool {
    if !state.changed() {
        return false;
    }
    let Some(path) = state_path() else {
        return false;
    };
    if let Some(d) = os::parent(&path) {
        if !os::exists(d) {
            os::mkdirs(d);
        }
    }
    os::write_atomic(&path, state.text().as_bytes())
}

fn load_config() -> Config {
    Config::parse(config_path().and_then(|p| os::read_text(&p)).as_deref())
}

enum Stage {
    State,
    Config,
    Session,
    Render,
}

const STAGE_COUNT: usize = Stage::Render as usize + 1;

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

fn compact_files(cwd: Option<&str>) -> [String; 3] {
    let main = os::claude_dir()
        .map(|d| os::join(&d, &["settings.json"]))
        .unwrap_or_default();
    let cwd = cwd.map(String::from).or_else(os::cwd);
    let project = |f: &str| {
        cwd.as_deref()
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
    let cfg = load_config();
    laps.lap(Stage::Config);
    let session = Session::parse(input);
    laps.lap(Stage::Session);
    let mut state = load_state();
    let project = session.project.as_deref().or(session.cwd.as_deref());
    let light = theme::detect_light(cfg.light_override, project, &mut state);
    let quotas = state.quotas(
        session.live,
        now,
        &state::account_tag(&os::claude_dir().unwrap_or_default()),
    );
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
    let sys = sample(&cfg, &mut state, now);
    laps.lap(Stage::State);
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

fn bench(runs: usize) {
    let input = demo_input(os::now());
    let target = state_path().map(|p| format!("{}.bench", p));
    let mut laps = Laps::new();
    let mut write = 0.0;
    let mut bytes = 0;
    let start = os::ticks();
    for _ in 0..runs {
        laps.last = os::ticks();
        let (out, state, _) = run(&input, os::now(), &mut laps);
        bytes = out.len();
        let write_start = os::ticks();
        if let Some(path) = &target {
            os::write_atomic(path, state.text().as_bytes());
        }
        write += os::ticks() - write_start;
    }
    let total = os::ticks() - start;
    if let Some(path) = &target {
        os::remove(path);
    }
    let per_run_ms = |seconds: f64| format!("{:.3}", seconds * MS_PER_SECOND / runs as f64);
    let mut report =
        messages::BENCH_SUMMARY.with(&[&runs.to_string(), &per_run_ms(total), &bytes.to_string()]);
    for (label, seconds) in [
        (messages::BENCH_STATE, laps.spent(Stage::State)),
        (messages::BENCH_CONFIG, laps.spent(Stage::Config)),
        (messages::BENCH_SESSION, laps.spent(Stage::Session)),
        (messages::BENCH_RENDER, laps.spent(Stage::Render)),
        (messages::BENCH_WRITE, write),
    ] {
        report.push_str(&format!(
            "  {:<10}{} ms\n",
            label.text(),
            per_run_ms(seconds)
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
        if os::write(&target, TEMPLATE.as_bytes()) {
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
            let light = theme::detect_light(cfg.light_override, os::cwd().as_deref(), &mut state);
            save_state(&state);
            os::out(if light { "light\n" } else { "dark\n" });
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

    #[test]
    fn the_written_template_passes_the_check() {
        assert!(check::problems(TEMPLATE).is_empty());
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
