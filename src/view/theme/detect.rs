use super::hosts::{host_files, host_source, host_stamp, mode::host_mode, Context, Host};
use super::locate::paths;
use super::locate::places::{var, Places};
use super::{cache, Decision, Mode, Source};
use crate::os;
use crate::state::State;

const VSCODE_WORKSPACE: [&str; 2] = [".vscode", "settings.json"];

fn host() -> Host {
    match os::env("TERM_PROGRAM").as_deref() {
        Some("vscode") => return Host::Vscode,
        Some("zed") => return Host::Zed,
        Some("ghostty") => return Host::Ghostty,
        _ => {}
    }
    if var("VSCODE_INJECTION").is_some() || var("VSCODE_PID").is_some() {
        return Host::Vscode;
    }
    if var("WT_SESSION").is_some() {
        return Host::Wt;
    }
    Host::Other
}

pub fn detect_light(
    light_override: Option<bool>,
    project: Option<&str>,
    state: &mut State,
    now: f64,
) -> bool {
    detect(light_override, project, state, now).light
}

pub fn detect(
    light_override: Option<bool>,
    project: Option<&str>,
    state: &mut State,
    now: f64,
) -> Decision {
    cache::drop_legacy(state);
    let background = os::env("STATUSLINE_BG");
    let colorfgbg = os::env("COLORFGBG");
    let chosen = Decision::chosen(background.as_deref(), light_override, colorfgbg.as_deref());
    if let Some(decision) = chosen {
        return decision;
    }
    let terminal = host();
    let places = Places::current(state);
    let order = paths::product_order();
    let files = host_files(terminal, &places, &order);
    let workspace = project
        .filter(|_| terminal == Host::Vscode)
        .map(|dir| os::join(dir, &VSCODE_WORKSPACE));
    let profile_id = var("WT_PROFILE_ID").filter(|_| terminal == Host::Wt);
    let context = Context {
        places: &places,
        order: &order,
        profile_id: profile_id.as_deref(),
        workspace: workspace.as_deref(),
    };
    let stamp = host_stamp(terminal, &files, &context);
    if let Some(decision) = cache::recall(state, &stamp, now) {
        return decision;
    }
    let (light, source) = match host_mode(terminal, &files, &context) {
        Mode::Light => (true, host_source(terminal)),
        Mode::Dark => (false, host_source(terminal)),
        Mode::System => (os::os_theme().unwrap_or(false), Source::System),
    };
    let decision = Decision { light, source };
    cache::remember(state, &stamp, decision, now);
    decision
}
