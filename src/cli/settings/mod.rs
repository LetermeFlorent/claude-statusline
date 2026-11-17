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
mod tests;
