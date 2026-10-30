#[cfg(any(target_os = "linux", test))]
mod linux;
#[cfg(not(windows))]
mod unix;
#[cfg(not(windows))]
pub use unix::*;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

use alloc::string::String;
use alloc::vec::Vec;

pub const MAX_READ_BYTES: usize = 1 << 20;
pub const CPU_SOURCE: u64 = 0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Activity {
    pub source: u64,
    pub clock: i64,
    pub busy: i64,
}

pub enum Claim {
    Held(Lock),
    Busy,
    Unavailable,
}

#[derive(Debug, PartialEq)]
pub enum Contents {
    Missing,
    Unreadable,
    Bytes(Vec<u8>),
}

pub fn join(base: &str, parts: &[&str]) -> String {
    let mut joined = String::from(base);
    for p in parts {
        if !joined.is_empty() && !joined.ends_with(SEP) && !joined.ends_with('/') {
            joined.push(SEP);
        }
        joined.push_str(p);
    }
    joined
}

pub fn read(path: &str) -> Option<Vec<u8>> {
    read_limited(path, MAX_READ_BYTES)
}

pub fn read_checked(path: &str, limit: usize) -> Contents {
    match read_limited(path, limit) {
        Some(data) => Contents::Bytes(data),
        None if exists(path) => Contents::Unreadable,
        None => Contents::Missing,
    }
}

pub fn read_text(path: &str) -> Option<String> {
    read(path).map(|b| String::from_utf8_lossy(&b).into_owned())
}

pub fn parent(path: &str) -> Option<&str> {
    path.rfind([SEP, '/']).map(|i| &path[..i])
}

pub fn file_name(path: &str) -> &str {
    path.rfind([SEP, '/']).map_or(path, |i| &path[i + 1..])
}

#[cfg(any(target_os = "linux", target_os = "macos", test))]
fn mount_holds(point: &str, path: &str) -> bool {
    point == "/"
        || path
            .strip_prefix(point)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

#[cfg(any(target_os = "linux", target_os = "macos", test))]
pub fn deepest_mount<T>(mounts: impl Iterator<Item = (String, T)>, path: &str) -> Option<T> {
    mounts
        .filter(|(point, _)| mount_holds(point, path))
        .max_by_key(|(point, _)| point.len())
        .map(|(_, mount)| mount)
}

pub fn fail(message: &str, code: i32) -> i32 {
    err(message);
    code
}

pub fn claude_dir() -> Option<String> {
    match env("CLAUDE_CONFIG_DIR") {
        Some(d) if !d.trim().is_empty() => Some(d),
        _ => home().map(|h| join(&h, &[".claude"])),
    }
}

#[cfg(test)]
mod tests;
