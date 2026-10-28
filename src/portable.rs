use crate::os;
use alloc::format;
use alloc::string::String;
use core::str::Chars;

const SHELL_SAFE_PUNCTUATION: &str = "_./:@%+=,-";
const DOUBLE_QUOTE_ESCAPES: &str = "$`\"\\";

fn slashes(path: &str) -> String {
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

fn tilde_is_home() -> bool {
    let Some(home) = os::home() else { return false };
    os::env("HOME")
        .filter(|h| !h.is_empty())
        .is_none_or(|h| same_path(&h, &home))
}

fn shell_home() -> Option<String> {
    os::env("HOME").filter(|h| !h.is_empty()).or_else(os::home)
}

fn expand(program: &str, home: Option<&str>) -> Option<String> {
    let full = match program.strip_prefix("~/") {
        Some(rest) => os::join(home?, &[rest]),
        None => String::from(program),
    };
    Some(slashes(&full))
}

pub fn expanded(program: &str) -> Option<String> {
    expand(program, shell_home().as_deref())
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

fn bare(c: char) -> bool {
    !c.is_ascii() || c.is_ascii_alphanumeric() || c == '~' || SHELL_SAFE_PUNCTUATION.contains(c)
}

fn single_quoted(chars: &mut Chars, word: &mut String) -> Option<()> {
    for c in chars.by_ref() {
        if c == '\'' {
            return Some(());
        }
        word.push(c);
    }
    None
}

fn double_quoted(chars: &mut Chars, word: &mut String) -> Option<()> {
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(()),
            '$' | '`' => return None,
            '\\' => {
                let escaped = chars.next()?;
                if !DOUBLE_QUOTE_ESCAPES.contains(escaped) {
                    word.push('\\');
                }
                word.push(escaped);
            }
            c => word.push(c),
        }
    }
    None
}

pub fn unquoted(command: &str) -> Option<String> {
    let mut word = String::new();
    let mut chars = command.trim().chars();
    while let Some(c) = chars.next() {
        match c {
            '\'' => single_quoted(&mut chars, &mut word)?,
            '"' => double_quoted(&mut chars, &mut word)?,
            '\\' => word.push(chars.next()?),
            c if bare(c) => word.push(c),
            _ => return None,
        }
    }
    (!word.is_empty()).then_some(word)
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
    fn quoted_programs_read_back_as_one_word() {
        for program in [
            "/opt/statusline",
            "/opt/my tools/statusline",
            "/opt/it's $HOME/statusline",
            "C:/Users/H\u{e9}l\u{e8}ne/.claude/bin/statusline.exe",
        ] {
            assert_eq!(unquoted(&quoted(program)).as_deref(), Some(program));
        }
    }

    #[test]
    fn shell_words_lose_their_quotes() {
        assert_eq!(
            unquoted(" ~/.claude/bin/statusline ").as_deref(),
            Some("~/.claude/bin/statusline")
        );
        assert_eq!(
            unquoted(r#""C:\Users\Jo\statusline.exe""#).as_deref(),
            Some(r"C:\Users\Jo\statusline.exe")
        );
        assert_eq!(
            unquoted(r#""/opt/a \"b\" \$c/statusline""#).as_deref(),
            Some(r#"/opt/a "b" $c/statusline"#)
        );
        assert_eq!(
            unquoted(r"/opt/a\ b/statusline").as_deref(),
            Some("/opt/a b/statusline")
        );
    }

    #[test]
    fn arguments_and_expansions_are_not_one_word() {
        for command in [
            "",
            "npx ccusage statusline",
            "/opt/statusline --demo",
            "'/opt/statusline",
            "\"/opt/statusline",
            "$HOME/statusline",
            "\"$HOME/statusline\"",
            "`which statusline`",
            "/opt/statusline;ls",
            "/opt/statusline\\",
        ] {
            assert_eq!(unquoted(command), None, "{}", command);
        }
    }

    #[test]
    fn a_tilde_expands_to_the_home_folder() {
        assert_eq!(
            expand("~/.claude/bin/statusline", Some("/home/a")).as_deref(),
            Some("/home/a/.claude/bin/statusline")
        );
        assert_eq!(
            expand("/opt/statusline", None).as_deref(),
            Some("/opt/statusline")
        );
        assert_eq!(expand("~/statusline", None), None);
        if cfg!(windows) {
            assert_eq!(
                expand("~/x.exe", Some("/c/Users/Jo")).as_deref(),
                Some("c:/Users/Jo/x.exe")
            );
            assert_eq!(
                expand(r"C:\Users\Jo\x.exe", None).as_deref(),
                Some("C:/Users/Jo/x.exe")
            );
        }
    }

    #[test]
    fn paths_match_whatever_the_trailing_slash() {
        assert!(same_path("/home/a/.claude/", "/home/a/.claude"));
        assert!(!same_path("/home/a/.claude", "/home/a/.claude-compte2"));
        if cfg!(windows) {
            assert!(same_path(r"C:\Users\Jo\.claude", "c:/users/jo/.claude/"));
            assert!(same_path("/c/Users/Jo", r"C:\Users\Jo"));
        }
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
