use crate::config::{background_word, hex};
use crate::json::{self, Value};
use crate::os;
use crate::state::State;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

// Fond clair ou sombre. L'ordre : STATUSLINE_BG, le reglage terminal_background,
// COLORFGBG, puis le terminal hote (VS Code et derives, Windows Terminal, Zed,
// Ghostty) avec le theme du systeme en repli, et sombre si rien ne repond.
// Le resultat de la detection est garde dans l'etat tant que les fichiers lus
// et le theme du systeme ne bougent pas.

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

const PRODUCTS: [&str; 5] = ["Code", "Code - Insiders", "VSCodium", "Cursor", "Windsurf"];

fn set(name: &str) -> bool {
    os::env(name).is_some_and(|v| !v.is_empty())
}

fn host() -> Host {
    match os::env("TERM_PROGRAM").as_deref() {
        Some("vscode") => return Host::Vscode,
        Some("zed") => return Host::Zed,
        Some("ghostty") => return Host::Ghostty,
        _ => {}
    }
    if set("VSCODE_INJECTION") || set("VSCODE_PID") {
        return Host::Vscode;
    }
    if set("WT_SESSION") {
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

fn name_mode(name: &str) -> Mode {
    if name.to_lowercase().contains("light") {
        Mode::Light
    } else {
        Mode::Dark
    }
}

// Cle a plat ("workbench.colorTheme") comme l'ecrit VS Code, sinon imbriquee
fn dotted<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    v.get(key).or_else(|| {
        let parts: Vec<&str> = key.split('.').collect();
        v.at(&parts)
    })
}

pub fn vscode(v: &Value) -> Mode {
    if dotted(v, "window.autoDetectColorScheme").and_then(|x| x.bool()) == Some(true) {
        return Mode::System;
    }
    match dotted(v, "workbench.colorTheme").and_then(|x| x.str()) {
        Some(t) => name_mode(t),
        None => Mode::Dark,
    }
}

const WT_LIGHT: [&str; 3] = ["One Half Light", "Solarized Light", "Tango Light"];

pub fn wt(v: &Value, profile_id: Option<&str>, os_light: &dyn Fn() -> bool) -> Mode {
    let (list, defaults) = match v.get("profiles") {
        Some(Value::Arr(a)) => (Some(a.as_slice()), None),
        Some(p) if p.is_obj() => (p.get("list").and_then(|l| l.arr()), p.get("defaults").filter(|d| d.is_obj())),
        _ => (None, None),
    };
    let wanted = profile_id.map(String::from).or_else(|| v.get("defaultProfile").and_then(|d| d.str()).map(String::from));
    let profile = match (list, wanted) {
        (Some(l), Some(id)) => l
            .iter()
            .find(|p| p.get("guid").and_then(|g| g.str()).is_some_and(|g| g.eq_ignore_ascii_case(&id))),
        _ => None,
    };
    let pick = |k: &str| profile.and_then(|p| p.get(k)).or_else(|| defaults.and_then(|d| d.get(k)));
    if let Some(bg) = hex(pick("background")) {
        return if luminance_light(bg) { Mode::Light } else { Mode::Dark };
    }
    let scheme = match pick("colorScheme") {
        Some(Value::Str(s)) => String::from(s.as_str()),
        Some(o) if o.is_obj() => {
            let light = match v.get("theme").and_then(|t| t.str()) {
                Some("light") => true,
                Some("dark") => false,
                _ => os_light(),
            };
            let k = if light { "light" } else { "dark" };
            String::from(o.get(k).and_then(|s| s.str()).unwrap_or("Campbell"))
        }
        _ => String::from("Campbell"),
    };
    let custom = v
        .get("schemes")
        .and_then(|s| s.arr())
        .and_then(|a| a.iter().find(|s| s.get("name").and_then(|n| n.str()) == Some(scheme.as_str())));
    if let Some(s) = custom {
        return match hex(s.get("background")) {
            Some(bg) if luminance_light(bg) => Mode::Light,
            _ => Mode::Dark,
        };
    }
    if WT_LIGHT.contains(&scheme.as_str()) {
        Mode::Light
    } else {
        Mode::Dark
    }
}

pub fn zed(v: &Value) -> Mode {
    match v.get("theme") {
        Some(Value::Str(s)) => name_mode(s),
        Some(o) if o.is_obj() => match o.get("mode").and_then(|m| m.str()) {
            Some("light") => Mode::Light,
            Some("dark") => Mode::Dark,
            _ => Mode::System,
        },
        _ => Mode::System,
    }
}

pub fn ghostty(text: &str) -> Mode {
    let mut theme: Option<Mode> = None;
    let mut bg: Option<Mode> = None;
    for line in text.lines() {
        let Some((k, val)) = line.split_once('=') else { continue };
        let val = val.trim().trim_matches('"');
        match k.trim() {
            "background" => {
                if let Some(c) = hex(Some(&Value::Str(String::from(val)))) {
                    bg = Some(if luminance_light(c) { Mode::Light } else { Mode::Dark });
                }
            }
            "theme" if !val.is_empty() => {
                theme = Some(if val.contains("light:") || val.contains("dark:") { Mode::System } else { name_mode(val) });
            }
            _ => {}
        }
    }
    bg.or(theme).unwrap_or(Mode::Dark)
}

fn config_home() -> Option<String> {
    match os::env("XDG_CONFIG_HOME") {
        Some(x) if !x.is_empty() => Some(x),
        _ => os::home().map(|h| os::join(&h, &[".config"])),
    }
}

fn vscode_files() -> Vec<String> {
    let base = if cfg!(windows) {
        os::env("APPDATA").filter(|s| !s.is_empty()).or_else(|| os::home().map(|h| os::join(&h, &["AppData", "Roaming"])))
    } else if cfg!(target_os = "macos") {
        os::home().map(|h| os::join(&h, &["Library", "Application Support"]))
    } else {
        config_home()
    };
    let Some(base) = base else { return Vec::new() };
    // Le produit qui a lance le terminal passe en premier
    let hint = ["VSCODE_GIT_ASKPASS_NODE", "VSCODE_GIT_ASKPASS_MAIN", "VSCODE_CWD"]
        .iter()
        .filter_map(|k| os::env(k))
        .map(|s| s.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    let first = if hint.contains("cursor") {
        3
    } else if hint.contains("windsurf") {
        4
    } else if hint.contains("vscodium") {
        2
    } else if hint.contains("insiders") {
        1
    } else {
        0
    };
    let mut order: Vec<usize> = alloc::vec![first];
    order.extend((0..PRODUCTS.len()).filter(|i| *i != first));
    order.iter().map(|i| os::join(&base, &[PRODUCTS[*i], "User", "settings.json"])).collect()
}

fn wt_files() -> Vec<String> {
    let mut out = Vec::new();
    let Some(local) = os::env("LOCALAPPDATA").filter(|s| !s.is_empty()).or_else(|| os::home().map(|h| os::join(&h, &["AppData", "Local"])))
    else {
        return out;
    };
    let packages = os::join(&local, &["Packages"]);
    let mut names = os::list(&packages, "Microsoft.WindowsTerminal");
    // La version stable avant Preview et Canary
    names.sort_by_key(|n| (n.len(), n.clone()));
    for n in names {
        out.push(os::join(&packages, &[&n, "LocalState", "settings.json"]));
    }
    out.push(os::join(&local, &["Microsoft", "Windows Terminal", "settings.json"]));
    out
}

fn zed_files() -> Vec<String> {
    let p = if cfg!(windows) {
        os::env("APPDATA").filter(|s| !s.is_empty()).map(|a| os::join(&a, &["Zed", "settings.json"]))
    } else {
        config_home().map(|c| os::join(&c, &["zed", "settings.json"]))
    };
    p.into_iter().collect()
}

fn ghostty_files() -> Vec<String> {
    let mut out: Vec<String> = config_home().map(|c| os::join(&c, &["ghostty", "config"])).into_iter().collect();
    if cfg!(target_os = "macos") {
        if let Some(h) = os::home() {
            out.push(os::join(&h, &["Library", "Application Support", "com.mitchellh.ghostty", "config"]));
        }
    }
    out
}

fn read_json(path: &str) -> Option<Value> {
    let t = os::read_text(path)?;
    json::parse(&t, true).filter(|v| v.is_obj())
}

fn host_mode(h: Host, files: &[String]) -> Mode {
    match h {
        Host::Vscode => files.iter().find_map(|f| read_json(f)).map(|v| vscode(&v)).unwrap_or(Mode::Dark),
        Host::Wt => {
            let id = os::env("WT_PROFILE_ID").filter(|s| !s.is_empty());
            let os_light = || os::os_theme().unwrap_or(false);
            files.iter().find_map(|f| read_json(f)).map(|v| wt(&v, id.as_deref(), &os_light)).unwrap_or(Mode::Dark)
        }
        Host::Zed => files.iter().find_map(|f| read_json(f)).map(|v| zed(&v)).unwrap_or(Mode::System),
        Host::Ghostty => {
            let mut m = Mode::Dark;
            for f in files {
                if let Some(t) = os::read_text(f) {
                    m = ghostty(&t);
                }
            }
            m
        }
        Host::Other => Mode::System,
    }
}

// true : fond clair
pub fn detect(config: Option<bool>, state: &mut State) -> bool {
    // Cle de la version 1
    state.remove("bg_at");
    if let Some(b) = os::env("STATUSLINE_BG").as_deref().and_then(background_word) {
        return b;
    }
    if let Some(b) = config {
        return b;
    }
    if let Some(b) = os::env("COLORFGBG").as_deref().and_then(colorfgbg) {
        return b;
    }
    let h = host();
    let files = match h {
        Host::Vscode => vscode_files(),
        Host::Wt => wt_files(),
        Host::Zed => zed_files(),
        Host::Ghostty => ghostty_files(),
        Host::Other => Vec::new(),
    };
    let mut stamp = String::from(match h {
        Host::Vscode => "vscode|",
        Host::Wt => "wt|",
        Host::Zed => "zed|",
        Host::Ghostty => "ghostty|",
        Host::Other => "os|",
    });
    for f in &files {
        match os::stamp(f) {
            Some((size, mtime)) => stamp.push_str(&format!("{}:{};", size, mtime)),
            None => stamp.push_str("0;"),
        }
    }
    stamp.push('|');
    stamp.push_str(&os::os_stamp());
    if state.get("bg_stamp") == Some(stamp.as_str()) {
        match state.get("bg_host") {
            Some("light") => return true,
            Some("dark") => return false,
            _ => {}
        }
    }
    let light = match host_mode(h, &files) {
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
        json::parse(s, true).unwrap()
    }

    #[test]
    fn terminaux() {
        assert_eq!(colorfgbg("0;15"), Some(true));
        assert_eq!(colorfgbg("15;0"), Some(false));
        assert_eq!(colorfgbg("15;default;7"), Some(true));
        assert_eq!(colorfgbg("0;-1"), None);
        assert_eq!(vscode(&j("{\"workbench.colorTheme\": \"Default Light Modern\"}")), Mode::Light);
        assert_eq!(vscode(&j("{\"workbench\": {\"colorTheme\": \"Monokai\"}}")), Mode::Dark);
        assert_eq!(vscode(&j("{\"window.autoDetectColorScheme\": true, \"workbench.colorTheme\": \"x\"}")), Mode::System);
        assert_eq!(vscode(&j("{}")), Mode::Dark);
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
        assert_eq!(wt(&j("{\"profiles\": {\"defaults\": {\"background\": \"#ffffff\"}}}"), None, &dark), Mode::Light);
        assert_eq!(zed(&j("{\"theme\": {\"mode\": \"system\"}}")), Mode::System);
        assert_eq!(zed(&j("{\"theme\": \"One Light\"}")), Mode::Light);
        assert_eq!(ghostty("theme = light:Builtin Light,dark:Builtin Dark\n"), Mode::System);
        assert_eq!(ghostty("background = #fdf6e3\ntheme = Dracula\n"), Mode::Light);
        assert_eq!(ghostty(""), Mode::Dark);
    }
}
