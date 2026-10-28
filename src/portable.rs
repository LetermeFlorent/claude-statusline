use crate::os;
use alloc::format;
use alloc::string::String;

const SHELL_SAFE_PUNCTUATION: &str = "_./:@%+=,-";

fn slashes(path: &str) -> String {
    if !cfg!(windows) {
        return String::from(path);
    }
    let s = path.replace('\\', "/");
    let b = s.as_bytes();
    let msys_drive = b.len() >= 2
        && b[0] == b'/'
        && b[1].is_ascii_alphabetic()
        && (b.len() == 2 || b[2] == b'/');
    if msys_drive {
        return format!("{}:{}", &s[1..2], &s[2..]);
    }
    s
}

pub fn same(a: &str, b: &str) -> bool {
    if cfg!(windows) {
        a.eq_ignore_ascii_case(b)
    } else {
        a == b
    }
}

pub fn under(path: &str, home: &str) -> Option<String> {
    let p = slashes(path);
    let h = slashes(home);
    let h = h.trim_end_matches('/');
    let head = p.get(..h.len())?;
    let rest = p.get(h.len()..)?.strip_prefix('/')?;
    if h.is_empty() || rest.is_empty() || !same(head, h) {
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

fn tilde_is_home() -> bool {
    let Some(home) = os::home() else { return false };
    let trimmed = |p: &str| String::from(slashes(p).trim_end_matches('/'));
    os::env("HOME")
        .filter(|h| !h.is_empty())
        .is_none_or(|h| same(&trimmed(&h), &trimmed(&home)))
}

fn shell_safe(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || SHELL_SAFE_PUNCTUATION.contains(c))
}

fn quoted(text: &str) -> String {
    if shell_safe(text) {
        return String::from(text);
    }
    format!("'{}'", text.replace('\'', r"'\''"))
}

pub fn command(exe: &str) -> String {
    match home_rest(exe).filter(|_| tilde_is_home()) {
        Some(rest) if shell_safe(&rest) => format!("~/{}", rest),
        _ => quoted(&slashes(exe)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsafe_paths_are_single_quoted() {
        assert_eq!(command("/opt/statusline"), "/opt/statusline");
        assert_eq!(
            command("/opt/my tools/statusline"),
            "'/opt/my tools/statusline'"
        );
        assert_eq!(
            command("/opt/it's $HOME/statusline"),
            r"'/opt/it'\''s $HOME/statusline'"
        );
        let backslash = if cfg!(windows) {
            "/opt/a/statusline"
        } else {
            r"'/opt/a\statusline'"
        };
        assert_eq!(command(r"/opt/a\statusline"), backslash);
    }

    #[cfg(not(windows))]
    #[test]
    fn awkward_folders_still_run_through_sh() {
        let sandbox = crate::sandbox::Sandbox::new("quote");
        let folder = sandbox.path("a b'c$d");
        std::fs::create_dir(&folder).unwrap();
        let program = os::join(&folder, &["statusline"]);
        std::os::unix::fs::symlink("/bin/echo", &program).unwrap();
        let line = format!("{} ok", command(&program));
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&line)
            .output()
            .unwrap();
        assert_eq!(output.stdout, b"ok\n");
    }

    #[test]
    fn chemins() {
        assert_eq!(
            under("/home/a/.claude/bin/statusline", "/home/a").as_deref(),
            Some(".claude/bin/statusline")
        );
        assert_eq!(
            under("/home/a/.claude/bin/statusline", "/home/a/").as_deref(),
            Some(".claude/bin/statusline")
        );
        assert_eq!(under("/home/ab/x", "/home/a"), None);
        assert_eq!(under("/opt/x", "/home/a"), None);
        assert_eq!(under("/home/a", "/home/a"), None);
        assert_eq!(under("/home/a/x", ""), None);
        assert_eq!(under("/x", "/"), None);
        if cfg!(windows) {
            assert_eq!(
                under(
                    "C:\\Users\\Jo\\.claude\\bin\\statusline.exe",
                    "c:\\users\\jo"
                )
                .as_deref(),
                Some(".claude/bin/statusline.exe")
            );
            assert_eq!(
                under("C:\\Users\\Jo\\x.exe", "/c/Users/Jo").as_deref(),
                Some("x.exe")
            );
            assert_eq!(under("D:\\Users\\Jo\\x.exe", "C:\\Users\\Jo"), None);
        } else {
            assert_eq!(under("/home/A/x", "/home/a"), None);
        }
    }
}
