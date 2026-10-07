#[cfg(not(windows))]
mod unix;
#[cfg(not(windows))]
pub use unix::*;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

use alloc::string::String;

pub fn join(base: &str, parts: &[&str]) -> String {
    let mut s = String::from(base);
    for p in parts {
        if !s.is_empty() && !s.ends_with(SEP) && !s.ends_with('/') {
            s.push(SEP);
        }
        s.push_str(p);
    }
    s
}

pub fn read_text(path: &str) -> Option<String> {
    read(path).map(|b| String::from_utf8_lossy(&b).into_owned())
}

pub fn parent(path: &str) -> Option<&str> {
    path.rfind([SEP, '/']).map(|i| &path[..i])
}

// Dossier de configuration du compte Claude Code en cours
pub fn claude_dir() -> Option<String> {
    match env("CLAUDE_CONFIG_DIR") {
        Some(d) if !d.trim().is_empty() => Some(d),
        _ => home().map(|h| join(&h, &[".claude"])),
    }
}
