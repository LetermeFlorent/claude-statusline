use crate::json::{self, Syntax, Value};
use crate::os;
use crate::portable;
use crate::settings::STATUS_LINE;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub const MAX_GLOBAL_BYTES: usize = 64 << 20;
const FILE_NAME: &str = ".claude.json";
const DEFAULT_DIR: &str = ".claude";
const PROGRAM_NAMES: [&str; 2] = ["statusline", "statusline.exe"];
const WINDOWS_PROGRAM_SUFFIX: &str = ".exe";

#[derive(Debug, PartialEq)]
pub enum Repair {
    Unreadable,
    Kept,
    Repaired { text: String, was: String },
}

pub fn candidates(claude_dir: &str, home: Option<&str>) -> Vec<String> {
    let mut paths = Vec::from([os::join(claude_dir, &[FILE_NAME])]);
    let default = |home: &&str| portable::same_path(claude_dir, &os::join(home, &[DEFAULT_DIR]));
    if let Some(home) = home.filter(default) {
        paths.push(os::join(home, &[FILE_NAME]));
    }
    paths
}

fn rooted(program: &str) -> bool {
    let b = program.as_bytes();
    let drive =
        b.len() > 2 && b[0].is_ascii_alphabetic() && b[1] == b':' && matches!(b[2], b'/' | b'\\');
    drive || program.starts_with('/') || program.starts_with("~/")
}

fn ours(program: &str) -> bool {
    let name = program.rsplit(['/', '\\']).next().unwrap_or(program);
    rooted(program)
        && PROGRAM_NAMES
            .iter()
            .any(|known| portable::same(name, known))
}

pub fn present(program: &str) -> bool {
    portable::expanded(program).is_none_or(|path| {
        os::exists(&path)
            || (cfg!(windows) && os::exists(&format!("{}{}", path, WINDOWS_PROGRAM_SUFFIX)))
    })
}

pub fn repair(text: &str, command: &str, is_present: impl Fn(&str) -> bool) -> Repair {
    let Some(mut config) = json::parse(text, Syntax::Strict).filter(Value::is_obj) else {
        return Repair::Unreadable;
    };
    let Some(mut line) = config.get(STATUS_LINE).filter(|l| l.is_obj()).cloned() else {
        return Repair::Kept;
    };
    let Some(was) = line.get("command").and_then(Value::str).map(String::from) else {
        return Repair::Kept;
    };
    let stale = portable::unquoted(&was).is_some_and(|p| ours(&p) && !is_present(&p));
    if !stale || was == command {
        return Repair::Kept;
    }
    line.set("command", Value::Str(String::from(command)));
    config.set(STATUS_LINE, line);
    Repair::Repaired {
        text: json::pretty(&config),
        was,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMMAND: &str = "~/.claude/bin/statusline";
    const STALE: &str = "C:/Users/old/.claude/bin/statusline.exe";

    fn with_command(command: &str) -> String {
        let mut line = Value::Obj(Vec::new());
        line.set("command", Value::Str(String::from(command)));
        let mut config = Value::Obj(Vec::new());
        config.set(STATUS_LINE, line);
        json::pretty(&config)
    }

    #[test]
    fn a_missing_statusline_program_is_replaced_in_place() {
        let text = concat!(
            "{\"numStartups\": 4, \"statusLine\": {\"type\": \"command\", ",
            "\"command\": \"C:/Users/old/.claude/bin/statusline.exe\", \"padding\": 0}, ",
            "\"projects\": {}}"
        );
        let expected = concat!(
            "{\n",
            "  \"numStartups\": 4,\n",
            "  \"statusLine\": {\n",
            "    \"type\": \"command\",\n",
            "    \"command\": \"~/.claude/bin/statusline\",\n",
            "    \"padding\": 0\n",
            "  },\n",
            "  \"projects\": {}\n",
            "}\n",
        );
        assert_eq!(
            repair(text, COMMAND, |_| false),
            Repair::Repaired {
                text: String::from(expected),
                was: String::from(STALE),
            }
        );
    }

    #[test]
    fn commands_that_run_something_else_are_kept() {
        for command in [
            "npx ccusage statusline",
            "~/.claude/statusline.sh",
            "statusline",
            "/opt/statusline --demo",
            "bash -c /opt/statusline",
        ] {
            let kept = repair(&with_command(command), COMMAND, |_| false);
            assert_eq!(kept, Repair::Kept, "{}", command);
        }
    }

    #[test]
    fn quoted_and_windows_paths_are_recognised() {
        for stale in [
            "'/home/old/my tools/statusline'",
            r#""C:\Users\old\.claude\bin\statusline.exe""#,
            "~/.claude/bin/statusline",
        ] {
            assert_eq!(
                repair(&with_command(stale), "/opt/statusline", |_| false),
                Repair::Repaired {
                    text: with_command("/opt/statusline"),
                    was: String::from(stale),
                },
                "{}",
                stale
            );
        }
    }

    #[test]
    fn a_program_still_on_disk_is_kept() {
        let quoted = with_command("'/opt/my tools/statusline'");
        let on_disk = |program: &str| program == "/opt/my tools/statusline";
        assert_eq!(repair(&quoted, COMMAND, on_disk), Repair::Kept);
    }

    #[test]
    fn the_current_command_is_kept() {
        assert_eq!(
            repair(&with_command(COMMAND), COMMAND, |_| false),
            Repair::Kept
        );
    }

    #[test]
    fn files_without_a_command_string_are_kept() {
        for text in [
            "{}",
            "{\"statusLine\": 3}",
            "{\"statusLine\": {\"command\": 3}}",
        ] {
            assert_eq!(repair(text, COMMAND, |_| false), Repair::Kept, "{}", text);
        }
    }

    #[test]
    fn files_that_are_not_one_strict_object_are_unreadable() {
        for text in ["", "{", "[]", "{\"statusLine\": {}} x", "{\"a\": 1,}"] {
            let unreadable = repair(text, COMMAND, |_| false);
            assert_eq!(unreadable, Repair::Unreadable, "{}", text);
        }
    }

    #[test]
    fn the_home_file_counts_only_for_the_default_folder() {
        let home = if cfg!(windows) {
            r"C:\Users\jo"
        } else {
            "/home/jo"
        };
        let default = os::join(home, &[".claude"]);
        let other = os::join(home, &[".claude-compte2"]);
        let inside = |dir: &str| os::join(dir, &[".claude.json"]);
        let slashed = format!("{}/", default);
        assert_eq!(
            candidates(&default, Some(home)),
            [inside(&default), inside(home)]
        );
        assert_eq!(
            candidates(&slashed, Some(home)),
            [inside(&slashed), inside(home)]
        );
        assert_eq!(candidates(&other, Some(home)), [inside(&other)]);
        assert_eq!(candidates(&default, None), [inside(&default)]);
    }
}
