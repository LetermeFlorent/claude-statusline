use crate::os;
use alloc::format;
use alloc::string::String;

pub fn slashes(path: &str) -> String {
    if !cfg!(windows) {
        return String::from(path);
    }
    let normalized = path.replace('\\', "/");
    let bytes = normalized.as_bytes();
    let msys_drive = bytes.len() >= 2
        && bytes[0] == b'/'
        && bytes[1].is_ascii_alphabetic()
        && (bytes.len() == 2 || bytes[2] == b'/');
    if msys_drive {
        return format!("{}:{}", &normalized[1..2], &normalized[2..]);
    }
    normalized
}

pub fn same(a: &str, b: &str) -> bool {
    if cfg!(windows) {
        a.eq_ignore_ascii_case(b)
    } else {
        a == b
    }
}

pub fn under(path: &str, home: &str) -> Option<String> {
    let candidate = slashes(path);
    let prefix = slashes(home);
    let prefix = prefix.trim_end_matches('/');
    let head = candidate.get(..prefix.len())?;
    let rest = candidate.get(prefix.len()..)?.strip_prefix('/')?;
    if prefix.is_empty() || rest.is_empty() || !same(head, prefix) {
        return None;
    }
    Some(String::from(rest))
}

#[cfg(not(windows))]
fn resolved(path: &str) -> Option<String> {
    std::fs::canonicalize(path)
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

#[cfg(windows)]
fn resolved(_: &str) -> Option<String> {
    None
}

pub fn home_rest(path: &str) -> Option<String> {
    let home = os::home()?;
    under(path, &home).or_else(|| under(path, &resolved(&home)?))
}

pub fn same_path(a: &str, b: &str) -> bool {
    same(
        slashes(a).trim_end_matches('/'),
        slashes(b).trim_end_matches('/'),
    )
}

pub fn tilde_is_home() -> bool {
    let Some(home) = os::home() else { return false };
    os::env("HOME")
        .filter(|h| !h.is_empty())
        .is_none_or(|h| same_path(&h, &home))
}

fn shell_home() -> Option<String> {
    os::env("HOME").filter(|h| !h.is_empty()).or_else(os::home)
}

pub(super) fn expand(program: &str, home: Option<&str>) -> Option<String> {
    let full = match program.strip_prefix("~/") {
        Some(rest) => os::join(home?, &[rest]),
        None => String::from(program),
    };
    Some(slashes(&full))
}

pub fn expanded(program: &str) -> Option<String> {
    expand(program, shell_home().as_deref())
}
