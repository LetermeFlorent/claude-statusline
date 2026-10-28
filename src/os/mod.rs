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

pub const MAX_READ_BYTES: usize = 1 << 20;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("statusline-os-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn files_over_one_mebibyte_are_refused() {
        let dir = scratch("size");
        let path = dir.join("settings.json");
        let text = path.to_string_lossy().into_owned();
        std::fs::write(&path, vec![b' '; MAX_READ_BYTES]).unwrap();
        let at_limit = read(&text).map(|data| data.len());
        std::fs::write(&path, vec![b' '; MAX_READ_BYTES + 1]).unwrap();
        let over_limit = read(&text).map(|data| data.len());
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(at_limit, Some(MAX_READ_BYTES));
        assert_eq!(over_limit, None);
    }

    #[test]
    fn devices_are_refused() {
        let device = if cfg!(windows) { "NUL" } else { "/dev/null" };
        assert_eq!(read(device), None);
    }

    #[cfg(not(windows))]
    #[test]
    fn fifo_is_refused_at_once() {
        let dir = scratch("fifo");
        let fifo = dir.join("settings.local.json");
        let made = std::process::Command::new("mkfifo").arg(&fifo).status();
        assert!(made.is_ok_and(|status| status.success()));
        let path = fifo.to_string_lossy().into_owned();
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(read(&path));
        });
        let result = receiver.recv_timeout(std::time::Duration::from_secs(2));
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(result, Ok(None));
    }
}
