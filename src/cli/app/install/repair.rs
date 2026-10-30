use super::edit::replace;
use crate::global_config::{self, Repair};
use crate::messages;
use crate::os::{self, Contents};
use alloc::string::String;

fn repair_global(path: &str, command: &str) -> Result<Option<String>, ()> {
    let raw = match os::read_checked(path, global_config::MAX_GLOBAL_BYTES) {
        Contents::Missing => return Ok(None),
        Contents::Unreadable => return Err(()),
        Contents::Bytes(raw) => raw,
    };
    let text = core::str::from_utf8(&raw).map_err(|_| ())?;
    match global_config::repair(text, command, global_config::present) {
        Repair::Kept => Ok(None),
        Repair::Unreadable => Err(()),
        Repair::Repaired { text: next, was } => match replace(path, &raw, &next) {
            Ok(told) => {
                os::out(&told);
                Ok(Some(was))
            }
            Err(message) => {
                os::err(&message);
                Err(())
            }
        },
    }
}

pub(super) fn repair_globals(command: &str) {
    let Some(dir) = os::claude_dir() else { return };
    for path in global_config::candidates(&dir, os::home().as_deref()) {
        match repair_global(&path, command) {
            Ok(Some(was)) => os::out(&messages::GLOBAL_REPAIRED.with(&[&path, &was])),
            Ok(None) => {}
            Err(()) => os::err(&messages::GLOBAL_KEPT.with(&[&path])),
        }
    }
}
