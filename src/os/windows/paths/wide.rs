use crate::os::windows::sys::*;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::ptr::null_mut;

const LEGACY_PATH_UNITS: usize = 248;
pub const VERBATIM_PREFIX: &str = "\\\\?\\";
const VERBATIM_UNC_PREFIX: &str = "\\\\?\\UNC\\";
const NT_PREFIX: &str = "\\??\\";
const UNC_PREFIX: &str = "\\\\";
pub const VERBATIM_UNC_NAME: &str = "UNC\\";
const MAX_BUFFER_UNITS: u32 = 1 << 20;

pub fn wide(s: &str) -> Vec<u16> {
    let mut v: Vec<u16> = s.encode_utf16().collect();
    v.push(0);
    v
}

pub fn verbatim(full: String) -> String {
    match full.strip_prefix(UNC_PREFIX) {
        Some(rest) if rest.starts_with(['?', '.']) => full,
        Some(rest) => format!("{}{}", VERBATIM_UNC_PREFIX, rest),
        None => format!("{}{}", VERBATIM_PREFIX, full),
    }
}

pub fn wide_path(path: &str) -> Vec<u16> {
    let plain = wide(path);
    let prefixed = path.starts_with(VERBATIM_PREFIX) || path.starts_with(NT_PREFIX);
    if prefixed || plain.len() < LEGACY_PATH_UNITS {
        return plain;
    }
    let full =
        with_growing_buffer(&|b, n| unsafe { GetFullPathNameW(plain.as_ptr(), n, b, null_mut()) });
    full.map_or(plain, |full| wide(&verbatim(full)))
}

pub unsafe fn units_until_nul<'a>(text: *const u16) -> &'a [u16] {
    let mut len = 0;
    while *text.add(len) != 0 {
        len += 1;
    }
    core::slice::from_raw_parts(text, len)
}

pub fn with_growing_buffer(call: &dyn Fn(*mut u16, u32) -> u32) -> Option<String> {
    let mut size = MAX_PATH;
    loop {
        let mut buf: Vec<u16> = vec![0; size as usize];
        let needed = call(buf.as_mut_ptr(), size);
        if needed == 0 {
            return None;
        }
        if needed < size {
            return Some(String::from_utf16_lossy(&buf[..needed as usize]));
        }
        size = if needed > size { needed + 1 } else { size * 2 };
        if size > MAX_BUFFER_UNITS {
            return None;
        }
    }
}
