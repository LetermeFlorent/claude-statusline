use crate::os::windows::paths::wide_path;
use crate::os::windows::sys::*;
use alloc::vec::Vec;
use core::ptr::null_mut;

const READ_GROWTH_BYTES: usize = 4 << 10;

pub fn open_existing(path: &str, access: u32, flags: u32) -> Handle {
    let wide_path = wide_path(path);
    unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            access,
            SHARE_ALL,
            null_mut(),
            OPEN_EXISTING,
            flags,
            0,
        )
    }
}

pub fn read_all(file: Handle, hint: usize, limit: usize) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(hint.saturating_add(1));
    while out.len() <= limit {
        if out.len() == out.capacity() {
            out.reserve(READ_GROWTH_BYTES);
        }
        let room = (out.capacity() - out.len())
            .min((limit - out.len()).saturating_add(1))
            .min(u32::MAX as usize);
        let mut got = 0u32;
        let ok = unsafe {
            ReadFile(
                file,
                out.as_mut_ptr().add(out.len()),
                room as u32,
                &mut got,
                null_mut(),
            )
        };
        if ok == 0 || got == 0 {
            break;
        }
        unsafe { out.set_len(out.len() + got as usize) };
    }
    out
}

pub fn read_limited(path: &str, limit: usize) -> Option<Vec<u8>> {
    let file = open_existing(path, GENERIC_READ, FILE_ATTRIBUTE_NORMAL);
    if file == INVALID {
        return None;
    }
    let mut size = 0i64;
    let on_disk = unsafe { GetFileType(file) } == FILE_TYPE_DISK
        && unsafe { GetFileSizeEx(file, &mut size) } != 0
        && (0..=limit as i64).contains(&size);
    let data = on_disk.then(|| read_all(file, size as usize, limit));
    unsafe { CloseHandle(file) };
    data.filter(|d| d.len() <= limit)
}
