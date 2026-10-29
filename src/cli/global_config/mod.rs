use crate::json::{self, Syntax, Value};
use crate::os;
use crate::portable;
use crate::settings::STATUS_LINE;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub const MAX_GLOBAL_BYTES: usize = 64 << 20;
const FILE_NAME: &str = ".claude.json";
const DEFAULT_DIR: &str = ".claude";
const PROGRAM_NAMES: [&str; 2] = ["statusline", "statusline.exe"];
const WINDOWS_PROGRAM_SUFFIX: &str = ".exe";

#[derive(Debug, PartialEq)]
pub enum Repair {
    Unreadable,
    Kept,
    Repaired { text: String, was: String },
}

pub fn candidates(claude_dir: &str, home: Option<&str>) -> Vec<String> {
    let mut paths = Vec::from([os::join(claude_dir, &[FILE_NAME])]);
    let default = |home: &&str| portable::same_path(claude_dir, &os::join(home, &[DEFAULT_DIR]));
    if let Some(home) = home.filter(default) {
        paths.push(os::join(home, &[FILE_NAME]));
    }
    paths
}

fn rooted(program: &str) -> bool {
    let bytes = program.as_bytes();
    let drive = bytes.len() > 2
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\');
    drive || program.starts_with('/') || program.starts_with("~/")
}

fn ours(program: &str) -> bool {
    let name = program.rsplit(['/', '\\']).next().unwrap_or(program);
    rooted(program)
        && PROGRAM_NAMES
            .iter()
            .any(|known| portable::same(name, known))
}

pub fn present(program: &str) -> bool {
    portable::expanded(program).is_none_or(|path| {
        os::exists(&path)
            || (cfg!(windows) && os::exists(&format!("{}{}", path, WINDOWS_PROGRAM_SUFFIX)))
    })
}

pub fn repair(text: &str, command: &str, is_present: impl Fn(&str) -> bool) -> Repair {
    let Some(mut config) = json::parse(text, Syntax::Strict).filter(Value::is_obj) else {
        return Repair::Unreadable;
    };
    let Some(mut line) = config.get(STATUS_LINE).filter(|l| l.is_obj()).cloned() else {
        return Repair::Kept;
    };
    let Some(was) = line.get("command").and_then(Value::str).map(String::from) else {
        return Repair::Kept;
    };
    let stale = portable::unquoted(&was).is_some_and(|p| ours(&p) && !is_present(&p));
    if !stale || was == command {
        return Repair::Kept;
    }
    line.set("command", Value::Str(String::from(command)));
    config.set(STATUS_LINE, line);
    Repair::Repaired {
        text: json::pretty(&config),
        was,
    }
}

#[cfg(test)]
mod tests;
