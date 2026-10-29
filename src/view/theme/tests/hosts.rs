use super::fixtures::*;
use crate::sandbox::Sandbox;
use crate::theme::hosts::{host_stamp, Context, Host};
use crate::theme::locate::places::{Places, Side};
use crate::theme::locate::wsl;
use crate::theme::Mode;

#[test]
fn a_light_windows_terminal_profile_is_read_under_a_fake_mnt_c() {
    let sandbox = Sandbox::new("wsl-terminal");
    let settings = format!("mnt/c/Users/alice/{}", wt_file(WT_PACKAGE));
    write(&sandbox, &settings, WT_LIGHT_DEFAULTS);
    let root = sandbox.dir().replace('\\', "/");
    let apps = format!("{root}/mnt/c/Users/alice/AppData/Local/Microsoft/WindowsApps");
    let profile = wsl::profile_in_path(["/usr/bin", apps.as_str()]);
    let found = places(profile.map(Side::windows), unix_side(&sandbox));
    assert_eq!(mode(Host::Wt, &found, None, None), Mode::Light);
    let lost = Places {
        unresolved: true,
        ..places(None, unix_side(&sandbox))
    };
    assert_eq!(mode(Host::Wt, &lost, None, None), Mode::System);
}

#[test]
fn a_second_ghostty_file_without_a_theme_keeps_the_first_one() {
    let sandbox = Sandbox::new("ghostty");
    let found = places(None, unix_side(&sandbox));
    let file = |name: &str, text: &str| write(&sandbox, &format!(".config/ghostty/{name}"), text);
    file("config.ghostty", "theme = Catppuccin Latte\n");
    file("config", "font-size = 13\n");
    assert_eq!(mode(Host::Ghostty, &found, None, None), Mode::Light);
    file("config.ghostty", "theme = Alabaster\n");
    file("themes/Alabaster", "background = #f7f7f7\n");
    assert_eq!(mode(Host::Ghostty, &found, None, None), Mode::Light);
    file("config", "theme = Dracula\n");
    assert_eq!(mode(Host::Ghostty, &found, None, None), Mode::Dark);
}

#[test]
fn vscode_takes_a_light_terminal_background_and_a_workspace_theme() {
    let sandbox = Sandbox::new("vscode");
    let found = places(None, unix_side(&sandbox));
    let user = ".config/Code/User/settings.json";
    write(&sandbox, user, VSCODE_LIGHT_TERMINAL);
    assert_eq!(mode(Host::Vscode, &found, None, None), Mode::Light);
    write(&sandbox, user, r#"{"workbench.colorTheme": "Monokai"}"#);
    assert_eq!(mode(Host::Vscode, &found, None, None), Mode::Dark);
    let extension = ".vscode/extensions/tonsky.alabaster-1.0.0/package.json";
    write(&sandbox, extension, ALABASTER_MANIFEST);
    let theme = r#"{"workbench.colorTheme": "Alabaster"}"#;
    let workspace = write(&sandbox, "project/.vscode/settings.json", theme);
    let with_workspace = mode(Host::Vscode, &found, None, Some(&workspace));
    assert_eq!(with_workspace, Mode::Light);
}

#[test]
fn windows_terminal_reads_the_file_of_the_profile_and_its_custom_theme() {
    let sandbox = Sandbox::new("wt");
    let found = places(Some(Side::windows(sandbox.dir())), None);
    let stable = write(&sandbox, &wt_file(WT_PACKAGE), WT_STABLE_SETTINGS);
    let preview = write(&sandbox, &wt_file(WT_PREVIEW), WT_PREVIEW_SETTINGS);
    assert_eq!(mode(Host::Wt, &found, Some("{A}"), None), Mode::Light);
    assert_eq!(mode(Host::Wt, &found, Some("{B}"), None), Mode::Dark);
    modified_ahead(&preview, 60);
    assert_eq!(mode(Host::Wt, &found, None, None), Mode::Dark);
    modified_ahead(&stable, 120);
    assert_eq!(mode(Host::Wt, &found, None, None), Mode::Light);
}

#[test]
fn an_unknown_host_is_dark_on_windows_and_follows_the_system_elsewhere() {
    let sandbox = Sandbox::new("other-host");
    let found = places(None, unix_side(&sandbox));
    let expected = if cfg!(windows) {
        Mode::Dark
    } else {
        Mode::System
    };
    assert_eq!(mode(Host::Other, &found, None, None), expected);
}

#[test]
fn zed_reads_the_appearance_of_a_custom_theme() {
    let sandbox = Sandbox::new("zed");
    let found = places(None, unix_side(&sandbox));
    let settings = r#"{"theme": "Paper"}"#;
    write(&sandbox, ".config/zed/settings.json", settings);
    assert_eq!(mode(Host::Zed, &found, None, None), Mode::Dark);
    let installed = ".local/share/zed/extensions/installed/paper/themes/paper.json";
    write(&sandbox, installed, PAPER_FAMILY);
    assert_eq!(mode(Host::Zed, &found, None, None), Mode::Light);
}

#[test]
fn each_windows_terminal_profile_has_its_own_cache_key() {
    let places = Places {
        windows: None,
        unix: None,
        macos: false,
        unresolved: false,
        ghostty_resources: None,
    };
    let key = |profile_id| {
        let context = Context {
            places: &places,
            order: &[],
            profile_id,
            workspace: None,
        };
        host_stamp(Host::Wt, &[], &context)
    };
    assert_ne!(key(Some("{a}")), key(Some("{b}")));
    assert_ne!(key(Some("{a}")), key(None));
}
