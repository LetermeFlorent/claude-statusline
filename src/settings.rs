use crate::json::{self, Syntax, Value};
use alloc::string::String;
use alloc::vec::Vec;

const STATUS_LINE: &str = "statusLine";
const REFRESH_SECONDS: &str = "3";

fn object(text: &str) -> Option<Value> {
    json::parse(text, Syntax::Strict).filter(|v| v.is_obj())
}

pub fn with_status_line(current: Option<&str>, command: &str) -> Option<String> {
    let mut settings = match current {
        Some(text) => object(text)?,
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

pub fn without_status_line(current: &str) -> Option<String> {
    let mut settings = object(current)?;
    if !settings.remove(STATUS_LINE) {
        return None;
    }
    Some(json::pretty(&settings))
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
            with_status_line(Some(" {} "), COMMAND).as_deref(),
            Some(ONLY_LINE)
        );
    }

    #[test]
    fn other_keys_keep_their_order() {
        let current = "{\"model\": \"opus\", \"env\": {\"B\": \"1\", \"A\": \"2\"}, \"hooks\": {}}";
        assert_eq!(
            with_status_line(Some(current), COMMAND).as_deref(),
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
            with_status_line(Some(current), COMMAND).as_deref(),
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
            with_status_line(Some("{\"a\": 1 // note\n}"), COMMAND),
            None
        );
        assert_eq!(with_status_line(Some("[1]"), COMMAND), None);
        assert_eq!(with_status_line(Some(""), COMMAND), None);
        assert_eq!(without_status_line("{\"statusLine\": {} /* x */}"), None);
        assert_eq!(without_status_line("[{\"statusLine\": {}}]"), None);
    }

    #[test]
    fn concatenated_objects_are_refused() {
        let current = "{\"permissions\":{}}{\"model\":\"opus\"}";
        assert_eq!(with_status_line(Some(current), COMMAND), None);
        assert_eq!(
            without_status_line("{\"statusLine\":{}}{\"model\":\"opus\"}"),
            None
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
                with_status_line(Some(current), COMMAND),
                None,
                "{}",
                current
            );
        }
    }

    #[test]
    fn nothing_to_remove_without_the_line() {
        assert_eq!(without_status_line("{\"model\": \"opus\"}"), None);
    }

    #[test]
    fn install_then_uninstall_gives_the_keys_back() {
        let current = "{\"model\": \"opus\", \"env\": {\"A\": \"1\"}, \"cleanupPeriodDays\": 30}";
        let installed = with_status_line(Some(current), COMMAND).unwrap();
        assert_eq!(
            without_status_line(&installed).as_deref(),
            Some("{\n  \"model\": \"opus\",\n  \"env\": {\n    \"A\": \"1\"\n  },\n  \"cleanupPeriodDays\": 30\n}\n")
        );
    }
}
