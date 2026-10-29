use crate::os::join;

pub fn args() -> Vec<String> {
    std::env::args_os()
        .map(|a| a.to_string_lossy().into_owned())
        .collect()
}

pub fn env(name: &str) -> Option<String> {
    std::env::var_os(name)
        .map(|v| v.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
}

pub fn pid() -> u32 {
    std::process::id()
}

pub fn exe() -> Option<String> {
    std::env::current_exe()
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

pub fn cwd() -> Option<String> {
    std::env::current_dir()
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

pub fn home() -> Option<String> {
    env("HOME").or_else(|| {
        std::env::home_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .filter(|h| !h.is_empty())
    })
}

pub fn state_dir() -> Option<String> {
    if cfg!(target_os = "macos") {
        return home().map(|h| join(&h, &["Library", "Application Support", "claude-statusline"]));
    }
    let base = env("XDG_STATE_HOME").or_else(|| home().map(|h| join(&h, &[".local", "state"])))?;
    Some(join(&base, &["claude-statusline"]))
}
