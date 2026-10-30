#[cfg(not(windows))]
use crate::os;
use crate::portable::shell::quoted;
use crate::portable::{command, unquoted};
#[cfg(not(windows))]
use alloc::format;

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
