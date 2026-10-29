use crate::compact;
use crate::config::{Config, Metric};
use crate::os;
use crate::render::SysValue;
use crate::state::State;
use alloc::string::String;
use alloc::vec::Vec;

pub(in crate::app) fn sample(cfg: &Config, state: &mut State, now: f64) -> Vec<SysValue> {
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

pub(in crate::app) fn local_only(dir: Option<&str>) -> Option<&str> {
    dir.filter(|dir| !os::on_network(dir))
}

pub(in crate::app) fn has_project_settings(folder: &str) -> bool {
    ["settings.json", "settings.local.json"]
        .iter()
        .any(|f| os::stamp(&os::join(folder, &[".claude", f])).is_some())
}

pub(in crate::app) fn compact_files(cwd: Option<&str>, project_dir: Option<&str>) -> [String; 3] {
    let main = os::claude_dir()
        .map(|d| os::join(&d, &["settings.json"]))
        .unwrap_or_default();
    let cwd = cwd.map(String::from).or_else(os::cwd);
    let folder = compact::files::project_folder(
        [cwd.as_deref(), project_dir],
        &|dir| local_only(Some(dir)).is_some(),
        &has_project_settings,
    );
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

pub(in crate::app) fn stat_file(p: &str) -> Option<(u64, i64)> {
    if p.is_empty() {
        None
    } else {
        os::stamp(p)
    }
}

pub(in crate::app) fn read_file(p: &str) -> Option<String> {
    if p.is_empty() {
        None
    } else {
        os::read_text(p)
    }
}
