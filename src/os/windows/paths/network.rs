use super::wide::{wide, VERBATIM_PREFIX, VERBATIM_UNC_NAME};
use crate::os::windows::sys::*;
use alloc::format;

fn is_separator(c: char) -> bool {
    c == '\\' || c == '/'
}

fn remote_drive(path: &str) -> bool {
    let mut start = path.chars();
    match (start.next(), start.next()) {
        (Some(letter), Some(':')) if letter.is_ascii_alphabetic() => {
            let root = wide(&format!("{}:\\", letter));
            unsafe { GetDriveTypeW(root.as_ptr()) == DRIVE_REMOTE }
        }
        _ => false,
    }
}

pub fn on_network(path: &str) -> bool {
    let (rest, unc) = match path.strip_prefix(VERBATIM_PREFIX) {
        Some(rest) => (
            rest,
            rest.get(..VERBATIM_UNC_NAME.len())
                .is_some_and(|name| name.eq_ignore_ascii_case(VERBATIM_UNC_NAME)),
        ),
        None => (
            path,
            path.starts_with(is_separator) && path[1..].starts_with(is_separator),
        ),
    };
    unc || remote_drive(rest)
}
