use crate::json::{self, Syntax, Value};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub const STATUS_LINE: &str = "statusLine";
pub const SUBAGENT_STATUS_LINE: &str = "subagentStatusLine";
const SUBAGENTS_ARGUMENT: &str = " subagents";
const PROGRAM_NAME: &str = "statusline";
const REFRESH_SECONDS: u8 = 3;
const TYPE_KEY: &str = "type";
const REFRESH_KEY: &str = "refreshInterval";
const COMMAND: &str = "command";
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

fn ours(entry: &Value) -> bool {
    let Some(command) = entry.get(COMMAND).and_then(Value::str) else {
        return false;
    };
    let Some(program) = command.strip_suffix(SUBAGENTS_ARGUMENT) else {
        return false;
    };
    let program = program.trim_end_matches(['\'', '"']);
    let name = program.rsplit(['/', '\\']).next().unwrap_or(program);
    name.strip_suffix(".exe").unwrap_or(name) == PROGRAM_NAME
}

fn set_command(entry: &mut Value, command: String) {
    entry.set(TYPE_KEY, Value::Str(String::from(COMMAND)));
    entry.set(COMMAND, Value::Str(command));
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
    set_command(&mut line, String::from(command));
    if line.get(REFRESH_KEY).is_none() {
        line.set(
            REFRESH_KEY,
            Value::Num(f64::from(REFRESH_SECONDS), REFRESH_SECONDS.to_string()),
        );
    }
    settings.set(STATUS_LINE, line);
    if settings.get(SUBAGENT_STATUS_LINE).is_none_or(ours) {
        let mut entry = Value::Obj(Vec::new());
        set_command(
            &mut entry,
            alloc::format!("{}{}", command, SUBAGENTS_ARGUMENT),
        );
        settings.set(SUBAGENT_STATUS_LINE, entry);
    }
    Some(json::pretty(&settings))
}

pub fn without_status_line(current: &[u8]) -> Removal {
    let Some(mut settings) = object(current) else {
        return Removal::Invalid;
    };
    let mine = settings.get(SUBAGENT_STATUS_LINE).is_some_and(ours);
    let removed_subagents = mine && settings.remove(SUBAGENT_STATUS_LINE);
    if !settings.remove(STATUS_LINE) && !removed_subagents {
        return Removal::NoLine;
    }
    Removal::Rewrite(json::pretty(&settings))
}

#[cfg(test)]
mod tests;
