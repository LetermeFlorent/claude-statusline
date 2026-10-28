use crate::config::{background_word, hex, hex_text};
use crate::json::{self, Syntax, Value};
use crate::os;
use crate::state::{fingerprint, State};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Reverse;
use places::{var, Places};

#[cfg(test)]
mod fixtures;
mod lookup;
mod names;
mod paths;
mod places;
mod wsl;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Light,
    Dark,
    System,
}

#[derive(Clone, Copy, PartialEq)]
enum Host {
    Vscode,
    Wt,
    Zed,
    Ghostty,
    Other,
}

type Lookup<'a> = dyn Fn(&str) -> Option<Mode> + 'a;

struct Context<'a> {
    places: &'a Places,
    order: &'a [usize],
    profile_id: Option<&'a str>,
    workspace: Option<&'a str>,
}

const WT_LIGHT: [&str; 3] = ["One Half Light", "Solarized Light", "Tango Light"];
const WT_DEFAULT_SCHEME: &str = "Campbell";
const TERMINAL_BACKGROUND: &str = "terminal.background";
const VSCODE_WORKSPACE: [&str; 2] = [".vscode", "settings.json"];
const LEGACY_BACKGROUND_KEY: &str = "bg_at";

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

pub fn colorfgbg(v: &str) -> Option<bool> {
    let last = v.rsplit(';').next()?.trim();
    let n: u32 = last.parse().ok()?;
    Some(n == 7 || n >= 9)
}

pub fn luminance_light(c: [f64; 3]) -> bool {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2] > 128.0
}

fn shade(color: [f64; 3]) -> Mode {
    if luminance_light(color) {
        Mode::Light
    } else {
        Mode::Dark
    }
}

fn color_mode(text: &str) -> Option<Mode> {
    let text = text.trim();
    let digits = text.strip_prefix('#').unwrap_or(text);
    let opaque = match digits.len() {
        8 => digits.get(..6),
        4 => digits.get(..3),
        _ => Some(digits),
    };
    hex_text(opaque?).map(shade)
}

fn named(name: &str, lookup: &Lookup<'_>) -> Mode {
    names::word_mode(name)
        .or_else(|| lookup(name))
        .unwrap_or(Mode::Dark)
}

fn flat_or_nested<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    v.get(key).or_else(|| {
        let parts: Vec<&str> = key.split('.').collect();
        v.at(&parts)
    })
}

fn setting<'a>(layers: &'a [Value], key: &str) -> Option<&'a Value> {
    layers.iter().find_map(|layer| flat_or_nested(layer, key))
}

fn terminal_background(layers: &[Value], theme: Option<&str>) -> Option<Mode> {
    let colors = setting(layers, "workbench.colorCustomizations")?;
    theme
        .and_then(|name| colors.get(&format!("[{}]", name)))
        .and_then(|scope| scope.get(TERMINAL_BACKGROUND))
        .or_else(|| colors.get(TERMINAL_BACKGROUND))
        .and_then(|color| color.str())
        .and_then(color_mode)
}

pub fn vscode(layers: &[Value], lookup: &Lookup<'_>) -> Mode {
    let theme = setting(layers, "workbench.colorTheme").and_then(|v| v.str());
    if let Some(mode) = terminal_background(layers, theme) {
        return mode;
    }
    if setting(layers, "window.autoDetectColorScheme").and_then(|v| v.bool()) == Some(true) {
        return Mode::System;
    }
    theme.map_or(Mode::Dark, |name| named(name, lookup))
}

fn wt_profiles(settings: &Value) -> (Option<&[Value]>, Option<&Value>) {
    match settings.get("profiles") {
        Some(Value::Arr(list)) => (Some(list.as_slice()), None),
        Some(profiles) if profiles.is_obj() => (
            profiles.get("list").and_then(|list| list.arr()),
            profiles
                .get("defaults")
                .filter(|defaults| defaults.is_obj()),
        ),
        _ => (None, None),
    }
}

fn wt_profile<'a>(settings: &'a Value, id: &str) -> Option<&'a Value> {
    wt_profiles(settings).0?.iter().find(|profile| {
        profile
            .get("guid")
            .and_then(|guid| guid.str())
            .is_some_and(|guid| guid.eq_ignore_ascii_case(id))
    })
}

fn app_theme(settings: &Value) -> Mode {
    let name = settings
        .get("theme")
        .and_then(|t| t.str())
        .unwrap_or("system");
    let custom = settings
        .get("themes")
        .and_then(|themes| lookup::item_named(themes, &["name"], name))
        .and_then(|theme| theme.at(&["window", "applicationTheme"]))
        .and_then(|v| v.str());
    match custom.unwrap_or(name) {
        "light" | "legacyLight" => Mode::Light,
        "dark" | "legacyDark" => Mode::Dark,
        _ => Mode::System,
    }
}

pub fn wt(settings: &Value, profile_id: Option<&str>, os_light: &dyn Fn() -> bool) -> Mode {
    let defaults = wt_profiles(settings).1;
    let wanted = profile_id.or_else(|| settings.get("defaultProfile").and_then(|d| d.str()));
    let profile = wanted.and_then(|id| wt_profile(settings, id));
    let pick = |key: &str| {
        profile
            .and_then(|p| p.get(key))
            .or_else(|| defaults.and_then(|d| d.get(key)))
    };
    if let Some(background) = hex(pick("background")) {
        return shade(background);
    }
    let scheme = match pick("colorScheme") {
        Some(Value::Str(name)) => name.as_str(),
        Some(pair) if pair.is_obj() => {
            let light = match app_theme(settings) {
                Mode::Light => true,
                Mode::Dark => false,
                Mode::System => os_light(),
            };
            let side = if light { "light" } else { "dark" };
            pair.get(side)
                .and_then(|s| s.str())
                .unwrap_or(WT_DEFAULT_SCHEME)
        }
        _ => WT_DEFAULT_SCHEME,
    };
    let custom = settings
        .get("schemes")
        .and_then(|schemes| lookup::item_named(schemes, &["name"], scheme));
    match custom {
        Some(custom) => hex(custom.get("background")).map_or(Mode::Dark, shade),
        None if WT_LIGHT.contains(&scheme) => Mode::Light,
        None => Mode::Dark,
    }
}

fn wt_settings(files: &[String], profile_id: Option<&str>) -> Option<Value> {
    let found: Vec<(Value, i64)> = files
        .iter()
        .filter_map(|file| Some((read_json(file)?, os::stamp(file)?.1)))
        .collect();
    let holder = profile_id.and_then(|id| {
        found
            .iter()
            .position(|(settings, _)| wt_profile(settings, id).is_some())
    });
    let newest = found
        .iter()
        .enumerate()
        .max_by_key(|(index, (_, mtime))| (*mtime, Reverse(*index)))
        .map(|(index, _)| index);
    let index = holder.or(newest)?;
    found.into_iter().nth(index).map(|(settings, _)| settings)
}

pub fn zed(settings: &Value, lookup: &Lookup<'_>) -> Mode {
    match settings.get("theme") {
        Some(Value::Str(name)) => named(name, lookup),
        Some(choice) if choice.is_obj() => match choice.get("mode").and_then(|m| m.str()) {
            Some("light") => Mode::Light,
            Some("dark") => Mode::Dark,
            _ => Mode::System,
        },
        _ => Mode::System,
    }
}

pub fn ghostty_keys(text: &str) -> (Option<Mode>, Option<&str>) {
    let mut background = None;
    let mut theme = None;
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"');
        match key.trim() {
            "background" if value.is_empty() => background = None,
            "background" => background = color_mode(value).or(background),
            "theme" if value.is_empty() => theme = None,
            "theme" => theme = Some(value),
            _ => {}
        }
    }
    (background, theme)
}

pub fn ghostty(text: &str, lookup: &Lookup<'_>) -> Mode {
    match ghostty_keys(text) {
        (Some(mode), _) => mode,
        (None, Some(pair)) if pair.contains("light:") || pair.contains("dark:") => Mode::System,
        (None, Some(name)) => named(name, lookup),
        (None, None) => Mode::Dark,
    }
}

fn read_json(path: &str) -> Option<Value> {
    let text = os::read_text(path)?;
    json::parse(&text, Syntax::Commented).filter(|v| v.is_obj())
}

fn host_files(terminal: Host, places: &Places, order: &[usize]) -> Vec<String> {
    match terminal {
        Host::Vscode => places.vscode_files(order),
        Host::Wt => places.wt_files(),
        Host::Zed => places.zed_files(),
        Host::Ghostty => places.ghostty_files(),
        Host::Other => Vec::new(),
    }
}

fn host_mode(terminal: Host, files: &[String], context: &Context<'_>) -> Mode {
    let places = context.places;
    let fallback = if places.unresolved {
        Mode::System
    } else {
        Mode::Dark
    };
    let first_json = || files.iter().find_map(|file| read_json(file));
    match terminal {
        Host::Vscode => {
            let layers: Vec<Value> = context
                .workspace
                .and_then(read_json)
                .into_iter()
                .chain(first_json())
                .collect();
            if layers.is_empty() {
                return fallback;
            }
            let roots = places.vscode_extensions(context.order);
            vscode(&layers, &|name| lookup::vscode_ui_theme(&roots, name))
        }
        Host::Wt => {
            let os_light = || os::os_theme().unwrap_or(false);
            wt_settings(files, context.profile_id).map_or(fallback, |settings| {
                wt(&settings, context.profile_id, &os_light)
            })
        }
        Host::Zed => first_json().map_or(Mode::System, |settings| {
            zed(&settings, &|name| {
                lookup::zed_appearance(&places.zed_theme_dirs(), name)
            })
        }),
        Host::Ghostty => {
            let texts: Vec<String> = files
                .iter()
                .filter_map(|file| os::read_text(file))
                .collect();
            let dirs = places.ghostty_theme_dirs();
            ghostty(&texts.join("\n"), &|name| {
                lookup::ghostty_theme(&dirs, name)
            })
        }
        Host::Other => Mode::System,
    }
}

fn host_stamp(terminal: Host, files: &[String], workspace: Option<&str>) -> String {
    let mut stamp = String::from(match terminal {
        Host::Vscode => "vscode|",
        Host::Wt => "wt|",
        Host::Zed => "zed|",
        Host::Ghostty => "ghostty|",
        Host::Other => "os|",
    });
    for file in files.iter().map(String::as_str).chain(workspace) {
        let (size, mtime) = os::stamp(file).unwrap_or((0, 0));
        stamp.push_str(&format!("{}:{}:{};", file, size, mtime));
    }
    stamp.push('|');
    stamp.push_str(&os::os_stamp());
    fingerprint([stamp.as_str()])
}

pub fn detect_light(
    light_override: Option<bool>,
    project: Option<&str>,
    state: &mut State,
) -> bool {
    state.remove(LEGACY_BACKGROUND_KEY);
    if let Some(b) = os::env("STATUSLINE_BG")
        .as_deref()
        .and_then(background_word)
    {
        return b;
    }
    if let Some(b) = light_override {
        return b;
    }
    if let Some(b) = os::env("COLORFGBG").as_deref().and_then(colorfgbg) {
        return b;
    }
    let terminal = host();
    let places = Places::current(state);
    let order = paths::product_order();
    let files = host_files(terminal, &places, &order);
    let workspace = project
        .filter(|_| terminal == Host::Vscode)
        .map(|dir| os::join(dir, &VSCODE_WORKSPACE));
    let stamp = host_stamp(terminal, &files, workspace.as_deref());
    if state.get("bg_stamp") == Some(stamp.as_str()) {
        match state.get("bg_host") {
            Some("light") => return true,
            Some("dark") => return false,
            _ => {}
        }
    }
    let profile_id = var("WT_PROFILE_ID");
    let context = Context {
        places: &places,
        order: &order,
        profile_id: profile_id.as_deref(),
        workspace: workspace.as_deref(),
    };
    let light = match host_mode(terminal, &files, &context) {
        Mode::Light => true,
        Mode::Dark => false,
        Mode::System => os::os_theme().unwrap_or(false),
    };
    state.set("bg_host", if light { "light" } else { "dark" });
    state.set("bg_stamp", &stamp);
    light
}

#[cfg(test)]
mod tests {
    use super::*;

    fn j(s: &str) -> Value {
        json::parse(s, Syntax::Commented).unwrap()
    }

    fn none(_: &str) -> Option<Mode> {
        None
    }

    #[test]
    fn terminal_settings_map_to_light_dark_or_system() {
        assert_eq!(colorfgbg("0;15"), Some(true));
        assert_eq!(colorfgbg("15;0"), Some(false));
        assert_eq!(colorfgbg("15;default;7"), Some(true));
        assert_eq!(colorfgbg("0;-1"), None);
        assert_eq!(
            vscode(
                &[j("{\"workbench.colorTheme\": \"Default Light Modern\"}")],
                &none
            ),
            Mode::Light
        );
        assert_eq!(
            vscode(
                &[j("{\"workbench\": {\"colorTheme\": \"Monokai\"}}")],
                &none
            ),
            Mode::Dark
        );
        assert_eq!(
            vscode(
                &[j(
                    "{\"window.autoDetectColorScheme\": true, \"workbench.colorTheme\": \"x\"}"
                )],
                &none
            ),
            Mode::System
        );
        assert_eq!(vscode(&[j("{}")], &none), Mode::Dark);
        let dark = || false;
        let light = || true;
        let s = "{\"profiles\": {\"defaults\": {\"colorScheme\": \"Mine\"}, \"list\": [{\"guid\": \"{A}\", \"colorScheme\": \"Tango Light\"}]},
                 \"schemes\": [{\"name\": \"Mine\", \"background\": \"#FAFAFA\"}, {\"name\": \"Tango Light\", \"background\": \"#000\"}],}";
        assert_eq!(wt(&j(s), None, &dark), Mode::Light);
        assert_eq!(wt(&j(s), Some("{a}"), &dark), Mode::Dark);
        assert_eq!(wt(&j("{}"), None, &dark), Mode::Dark);
        assert_eq!(wt(&j("{\"profiles\": [{\"guid\": \"g\", \"colorScheme\": \"One Half Light\"}], \"defaultProfile\": \"g\"}"), None, &dark), Mode::Light);
        let pair = "{\"profiles\": {\"defaults\": {\"colorScheme\": {\"light\": \"Solarized Light\", \"dark\": \"Campbell\"}}}}";
        assert_eq!(wt(&j(pair), None, &light), Mode::Light);
        assert_eq!(wt(&j(pair), None, &dark), Mode::Dark);
        let legacy = pair.replacen('{', "{\"theme\": \"legacyLight\", ", 1);
        assert_eq!(wt(&j(&legacy), None, &dark), Mode::Light);
        assert_eq!(
            wt(
                &j("{\"profiles\": {\"defaults\": {\"background\": \"#ffffff\"}}}"),
                None,
                &dark
            ),
            Mode::Light
        );
        assert_eq!(
            zed(&j("{\"theme\": {\"mode\": \"system\"}}"), &none),
            Mode::System
        );
        assert_eq!(zed(&j("{\"theme\": \"One Light\"}"), &none), Mode::Light);
        assert_eq!(
            ghostty("theme = light:Builtin Light,dark:Builtin Dark\n", &none),
            Mode::System
        );
        assert_eq!(
            ghostty("background = #fdf6e3\ntheme = Dracula\n", &none),
            Mode::Light
        );
        assert_eq!(ghostty("", &none), Mode::Dark);
    }

    #[test]
    fn vscode_reads_the_terminal_background_then_the_workspace_then_the_theme_file() {
        let scoped = r##"{"workbench.colorTheme": "Solarized Light", "workbench.colorCustomizations":
            {"[Solarized Light]": {"terminal.background": "#002b36"}, "terminal.background": "#ffffff"}}"##;
        assert_eq!(vscode(&[j(scoped)], &none), Mode::Dark);
        let workspace = j(r#"{"workbench.colorTheme": "Abyss"}"#);
        let user = j(r#"{"workbench.colorTheme": "Quiet Light"}"#);
        assert_eq!(vscode(&[workspace, user], &none), Mode::Dark);
        let alabaster = |name: &str| (name == "Alabaster").then_some(Mode::Light);
        let custom = j(r#"{"workbench.colorTheme": "Alabaster"}"#);
        assert_eq!(vscode(&[custom], &alabaster), Mode::Light);
    }

    #[test]
    fn colors_and_ghostty_keys_follow_the_last_valid_value() {
        assert_eq!(color_mode("#fdf6e3cc"), Some(Mode::Light));
        assert_eq!(color_mode("#fffa"), Some(Mode::Light));
        assert_eq!(color_mode("282c34"), Some(Mode::Dark));
        assert_eq!(color_mode("#12345"), None);
        assert_eq!(color_mode("abcde\u{e9}x"), None);
        let kept = ghostty_keys("background = #ffffff\nbackground = nope\n");
        assert_eq!(kept.0, Some(Mode::Light));
        assert_eq!(ghostty_keys("background = #ffffff\nbackground =\n").0, None);
        assert_eq!(ghostty_keys("theme = Dracula\ntheme = \"\"\n").1, None);
        assert_eq!(ghostty_keys("theme = Dracula\n").1, Some("Dracula"));
    }
}
