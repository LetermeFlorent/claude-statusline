use crate::os::windows::paths::wide_path;
use crate::os::windows::sys::*;
use crate::os::Claim;
use core::ptr::null_mut;

const LOCKED_BYTES: u32 = 1;
const MANUAL_RESET: i32 = 1;

pub struct Lock(Handle);

impl Drop for Lock {
    fn drop(&mut self) {
        unsafe {
            UnlockFile(self.0, 0, 0, LOCKED_BYTES, 0);
            CloseHandle(self.0);
        }
    }
}

pub fn lock(path: &str, wait_ms: u32) -> Claim {
    let wide_path = wide_path(path);
    let file = unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            SHARE_ALL,
            null_mut(),
            OPEN_ALWAYS,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
            0,
        )
    };
    if file == INVALID {
        return Claim::Unavailable;
    }
    let held = Lock(file);
    let event = unsafe { CreateEventW(null_mut(), MANUAL_RESET, 0, core::ptr::null()) };
    if event == 0 {
        return Claim::Unavailable;
    }
    let taken = wait_for_lock(file, event, wait_ms);
    unsafe { CloseHandle(event) };
    match taken {
        Some(true) => Claim::Held(held),
        Some(false) => Claim::Busy,
        None => Claim::Unavailable,
    }
}

fn wait_for_lock(file: Handle, event: Handle, wait_ms: u32) -> Option<bool> {
    let mut overlapped = Overlapped {
        event,
        ..Overlapped::default()
    };
    let mut moved = 0u32;
    unsafe {
        if LockFileEx(
            file,
            LOCKFILE_EXCLUSIVE_LOCK,
            0,
            LOCKED_BYTES,
            0,
            &mut overlapped,
        ) != 0
        {
            return Some(true);
        }
        if GetLastError() != ERROR_IO_PENDING {
            return None;
        }
        if GetOverlappedResultEx(file, &mut overlapped, &mut moved, wait_ms, 0) != 0 {
            return Some(true);
        }
        CancelIoEx(file, &mut overlapped);
        Some(GetOverlappedResultEx(file, &mut overlapped, &mut moved, INFINITE, 0) != 0)
    }
}
