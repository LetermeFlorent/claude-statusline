use crate::backup;
use crate::messages;
use crate::os::{self, Contents};
use crate::settings::{self, Removal};
use alloc::string::String;
use alloc::vec::Vec;

pub(in crate::cli::app) fn store(path: &str, text: &str) -> Result<(), String> {
    if os::write_atomic(path, text.as_bytes()) {
        Ok(())
    } else {
        Err(messages::CANNOT_WRITE.with(&[path]))
    }
}

pub(in crate::cli::app) fn replace(path: &str, raw: &[u8], text: &str) -> Result<String, String> {
    let saved =
        backup::save(path, raw, os::now()).map_err(|name| messages::CANNOT_WRITE.with(&[&name]))?;
    let told = messages::BACKUP.with(&[&saved]);
    match store(path, text) {
        Ok(()) => Ok(told),
        Err(message) => Err(told + &message),
    }
}

pub(in crate::cli::app) fn settings_bytes(path: &str) -> Result<Option<Vec<u8>>, String> {
    match os::read_checked(path, os::MAX_READ_BYTES) {
        Contents::Missing => Ok(None),
        Contents::Bytes(raw) => Ok(Some(raw)),
        Contents::Unreadable => Err(messages::SETTINGS_UNREADABLE.with(&[path])),
    }
}

pub(in crate::cli::app) fn add_line(path: &str, command: &str) -> Result<String, String> {
    let raw = settings_bytes(path)?;
    let next = settings::with_status_line(raw.as_deref(), command)
        .ok_or_else(|| messages::SETTINGS_INVALID.with(&[path]))?;
    let told = match raw {
        Some(raw) => replace(path, &raw, &next)?,
        None => {
            if let Some(d) = os::parent(path) {
                os::mkdirs(d);
            }
            store(path, &next)?;
            String::new()
        }
    };
    Ok(told + &messages::INSTALLED.with(&[path]))
}

pub(in crate::cli::app) fn remove_line(path: &str) -> Result<String, String> {
    let Some(raw) = settings_bytes(path)? else {
        return Ok(messages::NO_SETTINGS.with(&[path]));
    };
    let next = match settings::without_status_line(&raw) {
        Removal::Invalid => return Err(messages::SETTINGS_INVALID.with(&[path])),
        Removal::NoLine => return Ok(messages::NOTHING_TO_REMOVE.with(&[path])),
        Removal::Rewrite(next) => next,
    };
    let told = replace(path, &raw, &next)?;
    Ok(told + &messages::REMOVED.with(&[path]))
}
