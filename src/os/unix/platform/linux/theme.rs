use super::super::stamp_text;
use super::read_str;
use crate::os::{env, finish, home, join};
use std::io::Read;

#[cfg(target_os = "linux")]
fn config_home() -> Option<String> {
    env("XDG_CONFIG_HOME").or_else(|| home().map(|h| join(&h, &[".config"])))
}

const GSETTINGS_LIMIT: std::time::Duration = std::time::Duration::from_secs(1);

fn gsettings(key: &str) -> Option<String> {
    use std::process::{Command, Stdio};
    let mut child = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", key])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let status = finish(&mut child, GSETTINGS_LIMIT)?;
    if !status.success() {
        return None;
    }
    let mut text = Vec::new();
    child.stdout.take()?.read_to_end(&mut text).ok()?;
    Some(
        String::from_utf8_lossy(&text)
            .trim()
            .trim_matches('\'')
            .to_lowercase(),
    )
}

fn kde() -> Option<bool> {
    let desk = env("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_uppercase();
    if !desk.contains("KDE") {
        return None;
    }
    let text = read_str(&join(&config_home()?, &["kdeglobals"]))?;
    let mut section = "";
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            section = line;
            continue;
        }
        if section == "[Colors:Window]" {
            if let Some(v) = line.strip_prefix("BackgroundNormal=") {
                let channels: Vec<f64> =
                    v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                if channels.len() >= 3 {
                    return Some(crate::theme::luminance_light([
                        channels[0],
                        channels[1],
                        channels[2],
                    ]));
                }
            }
        }
    }
    None
}

pub fn os_theme() -> Option<bool> {
    if let Some(t) = env("GTK_THEME") {
        return Some(!t.to_lowercase().contains("dark"));
    }
    if let Some(b) = kde() {
        return Some(b);
    }
    match gsettings("color-scheme").as_deref() {
        Some("prefer-dark") => Some(false),
        Some("prefer-light") => Some(true),
        Some(_) => gsettings("gtk-theme").map(|t| !t.contains("dark")),
        None => None,
    }
}

pub fn os_stamp() -> String {
    let config_dir = config_home();
    format!(
        "{}|{}|{}|{}",
        env("GTK_THEME").unwrap_or_default(),
        env("XDG_CURRENT_DESKTOP").unwrap_or_default(),
        stamp_text(config_dir.as_ref().map(|c| join(c, &["dconf", "user"]))),
        stamp_text(config_dir.as_ref().map(|c| join(c, &["kdeglobals"]))),
    )
}
