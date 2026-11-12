use super::*;

const OTHER: &str = "{\"subagentStatusLine\": {\"type\": \"command\", \"command\": \"my-tool\"}}";

#[test]
fn a_foreign_subagent_line_is_kept() {
    let installed = with_status_line(Some(OTHER.as_bytes()), COMMAND).unwrap();
    assert!(installed.contains("\"my-tool\""), "{}", installed);
    assert!(!installed.contains("statusline subagents"), "{}", installed);
    assert!(installed.contains("\"statusLine\""), "{}", installed);
}

#[test]
fn uninstall_removes_ours_and_keeps_theirs() {
    let installed = with_status_line(Some(b"{}"), COMMAND).unwrap();
    assert_eq!(
        without_status_line(installed.as_bytes()),
        Removal::Rewrite(String::from("{}\n"))
    );
    let theirs = with_status_line(Some(OTHER.as_bytes()), COMMAND).unwrap();
    let Removal::Rewrite(left) = without_status_line(theirs.as_bytes()) else {
        panic!("expected a rewrite");
    };
    assert!(
        left.contains("my-tool") && !left.contains("\"statusLine\""),
        "{}",
        left
    );
    assert_eq!(without_status_line(OTHER.as_bytes()), Removal::NoLine);
    let only_subagents = "{\"subagentStatusLine\": {\"type\": \"command\", \"command\": \"C:\\\\x\\\\statusline.exe subagents\"}}";
    assert_eq!(
        without_status_line(only_subagents.as_bytes()),
        Removal::Rewrite(String::from("{}\n"))
    );
}

#[test]
fn update_refreshes_ours() {
    let old = with_status_line(Some(b"{}"), "/old/bin/statusline").unwrap();
    let next = with_status_line(Some(old.as_bytes()), "/new/bin/statusline").unwrap();
    assert!(next.contains("/new/bin/statusline subagents"), "{}", next);
    assert!(!next.contains("/old/"), "{}", next);
}
