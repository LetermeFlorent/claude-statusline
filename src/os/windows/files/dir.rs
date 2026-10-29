use super::stamp::exists;
use crate::os::windows::paths::wide_path;
use crate::os::windows::sys::*;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::ptr::null_mut;

const DRIVE_ROOT_INDEX: usize = 2;

pub fn mkdirs(path: &str) -> bool {
    for (i, c) in path.char_indices() {
        if (c == '\\' || c == '/') && i > DRIVE_ROOT_INDEX {
            let wide_parent = wide_path(&path[..i]);
            unsafe { CreateDirectoryW(wide_parent.as_ptr(), null_mut()) };
        }
    }
    let wide_path = wide_path(path);
    unsafe { CreateDirectoryW(wide_path.as_ptr(), null_mut()) };
    exists(path)
}

pub fn list(dir: &str, prefix: &str) -> Vec<String> {
    let mut out = Vec::new();
    let pattern = wide_path(&format!("{}\\{}*", dir, prefix));
    let mut data = FindData::new();
    let search = unsafe { FindFirstFileW(pattern.as_ptr(), &mut data) };
    if search == INVALID {
        return out;
    }
    loop {
        let name = data.name();
        if name != "." && name != ".." {
            out.push(name);
        }
        if unsafe { FindNextFileW(search, &mut data) } == 0 {
            break;
        }
    }
    unsafe { FindClose(search) };
    out
}
