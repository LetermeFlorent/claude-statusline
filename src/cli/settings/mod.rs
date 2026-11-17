use crate::json::{self, Syntax, Value};
use alloc::string::String;
use alloc::vec::Vec;

pub const STATUS_LINE: &str = "statusLine";
const SUBAGENT_STATUS_LINE: &str = "subagentStatusLine";
const OLD_SUBAGENTS_ARGUMENT: &str = " subagents";
const PROGRAM_NAME: &str = "statusline";
const ENV: &str = "env";
const PLUGIN_DIRS: &str = "CLAUDE_CODE_PLUGIN_DIRS";
const LIST_SEPARATOR: char = if cfg!(windows) { ';' } else { ':' };
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

fn old_subagent_line(entry: &Value) -> bool {
    let Some(command) = entry.get("command").and_then(Value::str) else {
        return false;
    };
    let Some(program) = command.strip_suffix(OLD_SUBAGENTS_ARGUMENT) else {
        return false;
    };
    let program = program.trim_end_matches(['\'', '"']);
    let name = program.rsplit(['/', '\\']).next().unwrap_or(program);
    name.strip_suffix(".exe").unwrap_or(name) == PROGRAM_NAME
}

fn drop_old_subagent_line(settings: &mut Value) -> bool {
    settings
        .get(SUBAGENT_STATUS_LINE)
        .is_some_and(old_subagent_line)
        && settings.remove(SUBAGENT_STATUS_LINE)
}

fn plugin_dirs(settings: &Value) -> Vec<String> {
    settings
        .at(&[ENV, PLUGIN_DIRS])
        .and_then(Value::str)
        .unwrap_or_default()
        .split(LIST_SEPARATOR)
        .filter(|d| !d.trim().is_empty())
        .map(String::from)
        .collect()
}

fn set_plugin_dirs(settings: &mut Value, dirs: &[String]) {
    let mut env = match settings.get(ENV) {
        Some(x) if x.is_obj() => x.clone(),
        _ => Value::Obj(Vec::new()),
    };
    if dirs.is_empty() {
        env.remove(PLUGIN_DIRS);
    } else {
        let mut list = String::new();
        for (i, d) in dirs.iter().enumerate() {
            if i > 0 {
                list.push(LIST_SEPARATOR);
            }
            list.push_str(d);
        }
        env.set(PLUGIN_DIRS, Value::Str(list));
    }
    if matches!(&env, Value::Obj(v) if v.is_empty()) {
        settings.remove(ENV);
    } else {
        settings.set(ENV, env);
    }
}

fn same_dir(a: &str, b: &str) -> bool {
    crate::portable::same_path(a.trim(), b.trim())
}

pub fn with_status_line(
    current: Option<&[u8]>,
    command: &str,
    plugin: Option<&str>,
) -> Option<String> {
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
    drop_old_subagent_line(&mut settings);
    if let Some(dir) = plugin {
        let mut dirs = plugin_dirs(&settings);
        if !dirs.iter().any(|d| same_dir(d, dir)) {
            dirs.push(String::from(dir));
            set_plugin_dirs(&mut settings, &dirs);
        }
    }
    Some(json::pretty(&settings))
}

pub fn without_status_line(current: &[u8], plugin: Option<&str>) -> Removal {
    let Some(mut settings) = object(current) else {
        return Removal::Invalid;
    };
    let old = drop_old_subagent_line(&mut settings);
    let mut unplugged = false;
    if let Some(dir) = plugin {
        let dirs = plugin_dirs(&settings);
        let kept: Vec<String> = dirs.iter().filter(|d| !same_dir(d, dir)).cloned().collect();
        if kept.len() != dirs.len() {
            set_plugin_dirs(&mut settings, &kept);
            unplugged = true;
        }
    }
    if !settings.remove(STATUS_LINE) && !old && !unplugged {
        return Removal::NoLine;
    }
    Removal::Rewrite(json::pretty(&settings))
}

#[cfg(test)]
mod tests;
