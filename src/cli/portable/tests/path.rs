use crate::portable::path::expand;
use crate::portable::path::under;
use crate::portable::same_path;

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
fn paths_strictly_inside_home_become_relative() {
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
