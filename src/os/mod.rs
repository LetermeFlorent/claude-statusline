#[cfg(not(windows))]
mod spawn_unix;
#[cfg(not(windows))]
pub use spawn_unix::*;
#[cfg(not(windows))]
mod unix;
#[cfg(not(windows))]
pub use unix::*;
#[cfg(windows)]
mod spawn_windows;
#[cfg(windows)]
pub use spawn_windows::*;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

use alloc::string::String;
use alloc::vec::Vec;

pub const MAX_READ_BYTES: usize = 1 << 20;

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

// Dossier de configuration du compte Claude Code en cours
pub fn claude_dir() -> Option<String> {
    match env("CLAUDE_CONFIG_DIR") {
        Some(d) if !d.trim().is_empty() => Some(d),
        _ => home().map(|h| join(&h, &[".claude"])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::Sandbox;
    #[cfg(not(windows))]
    use std::os::unix::fs::symlink;
    #[cfg(windows)]
    use std::os::windows::fs::symlink_file as symlink;

    #[test]
    fn files_over_one_mebibyte_are_refused() {
        let sandbox = Sandbox::new("size");
        let path = sandbox.path("settings.json");
        std::fs::write(&path, vec![b' '; MAX_READ_BYTES]).unwrap();
        let at_limit = read(&path).map(|data| data.len());
        std::fs::write(&path, vec![b' '; MAX_READ_BYTES + 1]).unwrap();
        let over_limit = read(&path).map(|data| data.len());
        assert_eq!(at_limit, Some(MAX_READ_BYTES));
        assert_eq!(over_limit, None);
    }

    #[test]
    fn a_larger_limit_reads_larger_files() {
        let sandbox = Sandbox::new("larger");
        let path = sandbox.path(".claude.json");
        std::fs::write(&path, vec![b' '; MAX_READ_BYTES + 1]).unwrap();
        let read_whole = read_limited(&path, 2 * MAX_READ_BYTES).map(|data| data.len());
        assert_eq!(read_whole, Some(MAX_READ_BYTES + 1));
    }

    #[test]
    fn a_file_too_large_to_read_is_not_a_missing_file() {
        let sandbox = Sandbox::new("contents");
        let path = sandbox.path("settings.json");
        assert_eq!(read_checked(&path, MAX_READ_BYTES), Contents::Missing);
        std::fs::write(&path, b"{}").unwrap();
        assert_eq!(read_checked(&path, 1), Contents::Unreadable);
        assert_eq!(read_checked(&path, 2), Contents::Bytes(b"{}".to_vec()));
    }

    #[test]
    fn children_start_in_the_given_folder() {
        let sandbox = Sandbox::new("cwd");
        #[cfg(windows)]
        let (shell, script) = (
            join(&system_dir().unwrap(), &["cmd.exe"]),
            ["/c", "cd>here.txt"],
        );
        #[cfg(not(windows))]
        let (shell, script) = (String::from("/bin/sh"), ["-c", "pwd>here.txt"]);
        assert_eq!(call(&shell, &script, &sandbox.dir(), 10), Some(0));
        assert_eq!(sandbox.files(), ["here.txt"]);
    }

    #[cfg(windows)]
    #[test]
    fn the_system_folder_holds_cmd() {
        assert!(exists(&join(&system_dir().unwrap(), &["cmd.exe"])));
    }

    #[test]
    fn devices_are_refused() {
        let device = if cfg!(windows) { "NUL" } else { "/dev/null" };
        assert_eq!(read(device), None);
    }

    #[cfg(not(windows))]
    #[test]
    fn fifo_is_refused_at_once() {
        let sandbox = Sandbox::new("fifo");
        let path = sandbox.path("settings.local.json");
        let made = std::process::Command::new("mkfifo").arg(&path).status();
        assert!(made.is_ok_and(|status| status.success()));
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(read(&path));
        });
        let result = receiver.recv_timeout(std::time::Duration::from_secs(2));
        assert_eq!(result, Ok(None));
    }

    #[cfg(not(windows))]
    #[test]
    fn atomic_write_keeps_the_mode_and_starts_private() {
        use std::os::unix::fs::PermissionsExt;
        const PERMISSION_BITS: u32 = 0o777;
        const SHARED_MODE: u32 = 0o640;
        let sandbox = Sandbox::new("mode");
        let (fresh, shared) = (sandbox.path("fresh.json"), sandbox.path("shared.json"));
        std::fs::write(&shared, b"{}").unwrap();
        std::fs::set_permissions(&shared, std::fs::Permissions::from_mode(SHARED_MODE)).unwrap();
        assert!(write_atomic(&fresh, b"{}"));
        assert!(write_atomic(&shared, b"[]"));
        let mode =
            |path: &str| std::fs::metadata(path).unwrap().permissions().mode() & PERMISSION_BITS;
        assert_eq!([mode(&fresh), mode(&shared)], [PRIVATE_MODE, SHARED_MODE]);
        assert_eq!(std::fs::read(&shared).unwrap(), b"[]");
    }

    #[test]
    fn atomic_write_clears_a_leftover_temp_file() {
        let sandbox = Sandbox::new("leftover");
        let path = sandbox.path("state.json");
        std::fs::write(format!("{}.{}.tmp", path, std::process::id()), b"{}").unwrap();
        assert!(write_atomic(&path, b"[]"));
        assert_eq!(std::fs::read(&path).unwrap(), b"[]");
        assert_eq!(sandbox.files(), ["state.json"]);
    }

    #[test]
    #[cfg_attr(windows, ignore = "creating a symbolic link needs an elevated token")]
    fn atomic_write_keeps_a_symbolic_link() {
        let sandbox = Sandbox::new("link");
        let (real, link) = (sandbox.path("real.json"), sandbox.path("settings.json"));
        std::fs::write(&real, b"{}").unwrap();
        symlink(&real, &link).unwrap();
        assert!(write_atomic(&link, b"[]"));
        assert!(std::fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(std::fs::read(&real).unwrap(), b"[]");
        assert_eq!(sandbox.files(), ["real.json", "settings.json"]);
    }
}
