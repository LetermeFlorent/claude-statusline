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
