use crate::json::{self, Syntax, Value};
use alloc::string::String;
use alloc::vec::Vec;

pub const STATUS_LINE: &str = "statusLine";
const REFRESH_SECONDS: &str = "3";
const BYTE_ORDER_MARK: char = '\u{feff}';

#[derive(Debug, PartialEq)]
pub enum Removal {
    Invalid,
    NoLine,
    Rewrite(String),
}

fn blank(text: &str) -> bool {
    text.strip_prefix(BYTE_ORDER_MARK)
        .unwrap_or(text)
        .bytes()
        .all(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
}

fn object(raw: &[u8]) -> Option<Value> {
    let text = core::str::from_utf8(raw).ok()?;
    if blank(text) {
        return Some(Value::Obj(Vec::new()));
    }
    json::parse(text, Syntax::Strict).filter(|v| v.is_obj())
}

pub fn with_status_line(current: Option<&[u8]>, command: &str) -> Option<String> {
    let mut settings = match current {
        Some(raw) => object(raw)?,
        None => Value::Obj(Vec::new()),
    };
    let mut line = match settings.get(STATUS_LINE) {
        Some(x) if x.is_obj() => x.clone(),
        _ => Value::Obj(Vec::new()),
    };
    line.set("type", Value::Str(String::from("command")));
    line.set("command", Value::Str(String::from(command)));
    if line.get("refreshInterval").is_none() {
        line.set(
            "refreshInterval",
            Value::Num(3.0, String::from(REFRESH_SECONDS)),
        );
    }
    settings.set(STATUS_LINE, line);
    Some(json::pretty(&settings))
}

pub fn without_status_line(current: &[u8]) -> Removal {
    let Some(mut settings) = object(current) else {
        return Removal::Invalid;
    };
    if !settings.remove(STATUS_LINE) {
        return Removal::NoLine;
    }
    Removal::Rewrite(json::pretty(&settings))
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMMAND: &str = "~/.claude/bin/statusline";
    const ONLY_LINE: &str = "{\n  \"statusLine\": {\n    \"type\": \"command\",\n    \"command\": \"~/.claude/bin/statusline\",\n    \"refreshInterval\": 3\n  }\n}\n";

    #[test]
    fn missing_file_gets_the_line() {
        assert_eq!(with_status_line(None, COMMAND).as_deref(), Some(ONLY_LINE));
    }

    #[test]
    fn empty_object_gets_the_line() {
        assert_eq!(
            with_status_line(Some(b" {} "), COMMAND).as_deref(),
            Some(ONLY_LINE)
        );
    }

    #[test]
    fn empty_file_gets_the_line() {
        for blank in ["", " \n", "\u{feff}"] {
            let installed = with_status_line(Some(blank.as_bytes()), COMMAND);
            assert_eq!(installed.as_deref(), Some(ONLY_LINE), "{:?}", blank);
        }
    }

    #[test]
    fn other_keys_keep_their_order() {
        let current = "{\"model\": \"opus\", \"env\": {\"B\": \"1\", \"A\": \"2\"}, \"hooks\": {}}";
        assert_eq!(
            with_status_line(Some(current.as_bytes()), COMMAND).as_deref(),
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
            with_status_line(Some(current.as_bytes()), COMMAND).as_deref(),
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
            with_status_line(Some(b"{\"a\": 1 // note\n}"), COMMAND),
            None
        );
        assert_eq!(with_status_line(Some(b"[1]"), COMMAND), None);
        assert_eq!(
            without_status_line(b"{\"statusLine\": {} /* x */}"),
            Removal::Invalid
        );
        assert_eq!(
            without_status_line(b"[{\"statusLine\": {}}]"),
            Removal::Invalid
        );
    }

    #[test]
    fn concatenated_objects_are_refused() {
        let current = "{\"permissions\":{}}{\"model\":\"opus\"}";
        assert_eq!(with_status_line(Some(current.as_bytes()), COMMAND), None);
        assert_eq!(
            without_status_line(b"{\"statusLine\":{}}{\"model\":\"opus\"}"),
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
                with_status_line(Some(current.as_bytes()), COMMAND),
                None,
                "{}",
                current
            );
        }
    }

    #[test]
    fn nothing_to_remove_without_the_line() {
        for current in [&b"{\"model\": \"opus\"}"[..], b"", b" \r\n"] {
            assert_eq!(without_status_line(current), Removal::NoLine);
        }
    }

    #[test]
    fn invalid_utf8_is_refused() {
        let current = b"{\"model\": \"op\xffus\"}";
        assert_eq!(with_status_line(Some(current), COMMAND), None);
        assert_eq!(without_status_line(current), Removal::Invalid);
    }

    #[test]
    fn install_then_uninstall_gives_the_keys_back() {
        let current = "{\"model\": \"opus\", \"env\": {\"A\": \"1\"}, \"cleanupPeriodDays\": 30}";
        let installed = with_status_line(Some(current.as_bytes()), COMMAND).unwrap();
        assert_eq!(
            without_status_line(installed.as_bytes()),
            Removal::Rewrite(String::from("{\n  \"model\": \"opus\",\n  \"env\": {\n    \"A\": \"1\"\n  },\n  \"cleanupPeriodDays\": 30\n}\n"))
        );
    }
}
