use crate::config::Config;
use crate::os;
use alloc::string::String;

pub(super) fn config_path() -> Option<String> {
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

pub(super) fn state_path() -> Option<String> {
    os::state_dir().map(|d| os::join(&d, &["state"]))
}

pub(super) fn settings_path() -> Option<String> {
    os::claude_dir().map(|d| os::join(&d, &["settings.json"]))
}

pub(super) fn load_config() -> Config {
    Config::parse(config_path().and_then(|p| os::read_text(&p)).as_deref())
}
