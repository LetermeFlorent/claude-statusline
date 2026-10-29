use super::stamp::exists;
use crate::os::windows::paths::wide_path;
use crate::os::windows::sys::*;
use core::ptr::null_mut;

const MAX_WRITE_BYTES: usize = 1 << 30;

pub fn write_handle(file: Handle, data: &[u8]) -> bool {
    let mut done = 0usize;
    while done < data.len() {
        let mut written = 0u32;
        let len = core::cmp::min(data.len() - done, MAX_WRITE_BYTES) as u32;
        let ok = unsafe { WriteFile(file, data.as_ptr().add(done), len, &mut written, null_mut()) };
        if ok == 0 || written == 0 {
            return false;
        }
        done += written as usize;
    }
    true
}

fn write_with(path: &str, share: u32, disposition: u32, data: &[u8]) -> bool {
    let wide_path = wide_path(path);
    let file = unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            GENERIC_WRITE,
            share,
            null_mut(),
            disposition,
            FILE_ATTRIBUTE_NORMAL,
            0,
        )
    };
    if file == INVALID {
        return false;
    }
    let ok = write_handle(file, data);
    unsafe { CloseHandle(file) };
    ok
}

pub fn write(path: &str, data: &[u8]) -> bool {
    write_with(path, FILE_SHARE_READ, CREATE_ALWAYS, data)
}

pub fn remove(path: &str) -> bool {
    let wide_path = wide_path(path);
    unsafe { DeleteFileW(wide_path.as_ptr()) != 0 }
}

pub fn create_private(path: &str, data: &[u8]) -> bool {
    write_with(path, SHARE_NONE, CREATE_NEW, data)
}

pub fn make_executable(path: &str) -> bool {
    exists(path)
}

pub fn hard_link(existing: &str, link: &str) -> bool {
    let (wide_existing, wide_link) = (wide_path(existing), wide_path(link));
    unsafe { CreateHardLinkW(wide_link.as_ptr(), wide_existing.as_ptr(), null_mut()) != 0 }
}
