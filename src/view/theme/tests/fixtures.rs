use crate::os;
use crate::sandbox::Sandbox;
use crate::theme::hosts::mode::host_mode;
use crate::theme::hosts::{host_files, Context, Host};
use crate::theme::locate::places::{Places, Side};
use crate::theme::Mode;
use std::fs::{self, File};
use std::time::{Duration, SystemTime};

const ORDER: [usize; 5] = [0, 1, 2, 3, 4];
pub const WT_PACKAGE: &str = "Microsoft.WindowsTerminal_8wekyb3d8bbwe";
pub const WT_PREVIEW: &str = "Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe";
pub const WT_LIGHT_DEFAULTS: &str =
    r#"{"profiles": {"defaults": {"colorScheme": "One Half Light"}}}"#;
pub const WT_STABLE_SETTINGS: &str = r#"{"theme": "Paper", "defaultProfile": "{a}",
    "themes": [{"name": "Paper", "window": {"applicationTheme": "light"}}],
    "profiles": {"list": [{"guid": "{a}", "colorScheme": {"light": "Tango Light", "dark": "Campbell"}}]}}"#;
pub const WT_PREVIEW_SETTINGS: &str =
    r#"{"profiles": {"list": [{"guid": "{b}", "colorScheme": "Campbell"}]}}"#;
pub const VSCODE_LIGHT_TERMINAL: &str = r##"{"workbench.colorTheme": "Monokai",
    "workbench.colorCustomizations": {"terminal.background": "#FDF6E3"}}"##;
pub const ALABASTER_MANIFEST: &str =
    r#"{"contributes": {"themes": [{"label": "Alabaster", "uiTheme": "vs"}]}}"#;
pub const PAPER_FAMILY: &str = r#"{"themes": [{"name": "Paper", "appearance": "light"}]}"#;

pub fn write(sandbox: &Sandbox, relative: &str, text: &str) -> String {
    let path = sandbox.path(relative);
    fs::create_dir_all(os::parent(&path).unwrap()).unwrap();
    fs::write(&path, text).unwrap();
    path
}

pub fn wt_file(package: &str) -> String {
    format!("AppData/Local/Packages/{package}/LocalState/settings.json")
}

pub fn places(windows: Option<Side>, unix: Option<Side>) -> Places {
    Places {
        windows,
        unix,
        macos: false,
        unresolved: false,
        ghostty_resources: None,
    }
}

pub fn unix_side(sandbox: &Sandbox) -> Option<Side> {
    let home = sandbox.dir();
    Some(Side {
        config: os::join(&home, &[".config"]),
        data: os::join(&home, &[".local", "share"]),
        home,
    })
}

pub fn mode(host: Host, places: &Places, id: Option<&str>, workspace: Option<&str>) -> Mode {
    let files = host_files(host, places, &ORDER);
    let context = Context {
        places,
        order: &ORDER,
        profile_id: id,
        workspace,
    };
    host_mode(host, &files, &context)
}

pub fn modified_ahead(path: &str, seconds: u64) {
    let file = File::options().write(true).open(path).unwrap();
    let later = SystemTime::now() + Duration::from_secs(seconds);
    file.set_modified(later).unwrap();
}
