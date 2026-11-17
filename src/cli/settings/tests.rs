use super::*;

const COMMAND: &str = "~/.claude/bin/statusline";
const ONLY_LINE: &str = "{\n  \"statusLine\": {\n    \"type\": \"command\",\n    \"command\": \"~/.claude/bin/statusline\",\n    \"refreshInterval\": 3\n  }\n}\n";

#[test]
fn missing_file_gets_the_line() {
    assert_eq!(
        with_status_line(None, COMMAND, None).as_deref(),
        Some(ONLY_LINE)
    );
}

#[test]
fn empty_object_gets_the_line() {
    assert_eq!(
        with_status_line(Some(b" {} "), COMMAND, None).as_deref(),
        Some(ONLY_LINE)
    );
}

#[test]
fn empty_file_gets_the_line() {
    for blank in ["", " \n", "\u{feff}"] {
        let installed = with_status_line(Some(blank.as_bytes()), COMMAND, None);
        assert_eq!(installed.as_deref(), Some(ONLY_LINE), "{:?}", blank);
    }
}

#[test]
fn other_keys_keep_their_order() {
    let current = "{\"model\": \"opus\", \"env\": {\"B\": \"1\", \"A\": \"2\"}, \"hooks\": {}}";
    assert_eq!(
        with_status_line(Some(current.as_bytes()), COMMAND, None).as_deref(),
        Some(concat!(
            "{\n",
            "  \"model\": \"opus\",\n",
            "  \"env\": {\n    \"B\": \"1\",\n    \"A\": \"2\"\n  },\n",
            "  \"hooks\": {},\n",
            "  \"statusLine\": {\n    \"type\": \"command\",\n    \"command\": \"~/.claude/bin/statusline\",\n    \"refreshInterval\": 3\n  }\n",
            "}\n",
        ))
    );
}

#[test]
fn existing_line_is_replaced_in_place() {
    let current = "{\"a\": 1, \"statusLine\": {\"type\": \"static\", \"command\": \"old\", \"refreshInterval\": 10, \"padding\": 0}, \"z\": 2}";
    assert_eq!(
        with_status_line(Some(current.as_bytes()), COMMAND, None).as_deref(),
        Some(concat!(
            "{\n",
            "  \"a\": 1,\n",
            "  \"statusLine\": {\n    \"type\": \"command\",\n    \"command\": \"~/.claude/bin/statusline\",\n    \"refreshInterval\": 10,\n    \"padding\": 0\n  },\n",
            "  \"z\": 2\n",
            "}\n",
        ))
    );
}

#[test]
fn jsonc_and_array_roots_are_refused() {
    assert_eq!(
        with_status_line(Some(b"{\"a\": 1 // note\n}"), COMMAND, None),
        None
    );
    assert_eq!(with_status_line(Some(b"[1]"), COMMAND, None), None);
    assert_eq!(
        without_status_line(b"{\"statusLine\": {} /* x */}", None),
        Removal::Invalid
    );
    assert_eq!(
        without_status_line(b"[{\"statusLine\": {}}]", None),
        Removal::Invalid
    );
}

#[test]
fn concatenated_objects_are_refused() {
    let current = "{\"permissions\":{}}{\"model\":\"opus\"}";
    assert_eq!(
        with_status_line(Some(current.as_bytes()), COMMAND, None),
        None
    );
    assert_eq!(
        without_status_line(b"{\"statusLine\":{}}{\"model\":\"opus\"}", None),
        Removal::Invalid
    );
}

#[test]
fn trailing_text_and_comma_mistakes_are_refused() {
    for current in [
        "{\"model\": \"opus\"} x",
        "{\"model\": \"opus\" \"env\": {}}",
        "{\"env\": {\"A\": [1 2]}}",
        "{\"model\": \"opus\",}",
        "{,\"model\": \"opus\"}",
        "{\"env\": [1,,2]}",
    ] {
        assert_eq!(
            with_status_line(Some(current.as_bytes()), COMMAND, None),
            None,
            "{}",
            current
        );
    }
}

#[test]
fn nothing_to_remove_without_the_line() {
    for current in [&b"{\"model\": \"opus\"}"[..], b"", b" \r\n"] {
        assert_eq!(without_status_line(current, None), Removal::NoLine);
    }
}

#[test]
fn invalid_utf8_is_refused() {
    let current = b"{\"model\": \"op\xffus\"}";
    assert_eq!(with_status_line(Some(current), COMMAND, None), None);
    assert_eq!(without_status_line(current, None), Removal::Invalid);
}

#[test]
fn install_then_uninstall_gives_the_keys_back() {
    let current = "{\"model\": \"opus\", \"env\": {\"A\": \"1\"}, \"cleanupPeriodDays\": 30}";
    let installed = with_status_line(Some(current.as_bytes()), COMMAND, None).unwrap();
    assert_eq!(
        without_status_line(installed.as_bytes(), None),
        Removal::Rewrite(String::from("{\n  \"model\": \"opus\",\n  \"env\": {\n    \"A\": \"1\"\n  },\n  \"cleanupPeriodDays\": 30\n}\n"))
    );
}

#[test]
fn install_adds_the_plugin_folder_once_and_drops_the_old_subagent_line() {
    let sep = if cfg!(windows) { ';' } else { ':' };
    let current = "{\"env\": {\"A\": \"1\", \"CLAUDE_CODE_PLUGIN_DIRS\": \"/other\"}, \"subagentStatusLine\": {\"type\": \"command\", \"command\": \"~/.claude/bin/statusline.exe subagents\"}}";
    let installed = with_status_line(Some(current.as_bytes()), COMMAND, Some("~/p")).unwrap();
    assert!(installed.contains(&format!(
        "\"CLAUDE_CODE_PLUGIN_DIRS\": \"/other{}~/p\"",
        sep
    )));
    assert!(installed.contains("\"A\": \"1\""));
    assert!(!installed.contains("subagentStatusLine"));
    let again = with_status_line(Some(installed.as_bytes()), COMMAND, Some("~/p")).unwrap();
    assert_eq!(again, installed);
    let Removal::Rewrite(removed) = without_status_line(installed.as_bytes(), Some("~/p")) else {
        panic!("nothing removed");
    };
    assert!(removed.contains("\"CLAUDE_CODE_PLUGIN_DIRS\": \"/other\""));
    assert!(!removed.contains("statusLine"));
}

#[test]
fn uninstall_drops_an_env_block_it_emptied_and_keeps_a_foreign_subagent_line() {
    let current = "{\"subagentStatusLine\": {\"type\": \"command\", \"command\": \"mine\"}}";
    let installed = with_status_line(Some(current.as_bytes()), COMMAND, Some("~/p")).unwrap();
    assert!(installed.contains("\"command\": \"mine\""));
    let Removal::Rewrite(removed) = without_status_line(installed.as_bytes(), Some("~/p")) else {
        panic!("nothing removed");
    };
    assert!(!removed.contains("\"env\""));
    assert!(removed.contains("\"command\": \"mine\""));
}
