use super::read::open_existing;
use super::write::{remove, write};
use crate::os::windows::paths::{wide_path, with_growing_buffer};
use crate::os::windows::process::pid;
use crate::os::windows::sys::*;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

const RENAME_TRIES: u32 = 3;
const RENAME_RETRY_MS: u32 = 5;

fn resolved(path: &str) -> String {
    let wide_path = wide_path(path);
    let attributes = unsafe { GetFileAttributesW(wide_path.as_ptr()) };
    let linked = attributes != u32::MAX && attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0;
    let file = if linked {
        open_existing(path, GENERIC_READ, FILE_ATTRIBUTE_NORMAL)
    } else {
        INVALID
    };
    if file == INVALID {
        return String::from(path);
    }
    let target = with_growing_buffer(&|b, n| unsafe {
        GetFinalPathNameByHandleW(file, b, n, VOLUME_NAME_DOS)
    });
    unsafe { CloseHandle(file) };
    target.unwrap_or_else(|| String::from(path))
}

pub fn write_atomic(path: &str, data: &[u8]) -> bool {
    let target = resolved(path);
    let tmp = format!("{}.{}.tmp", target, pid());
    let swapped = write(&tmp, data)
        && (0..RENAME_TRIES).any(|attempt| {
            if attempt > 0 {
                crate::os::sleep(RENAME_RETRY_MS);
            }
            rename(&tmp, &target)
        });
    if !swapped {
        remove(&tmp);
    }
    swapped
}

pub fn rename(from: &str, to: &str) -> bool {
    let (wide_from, wide_to) = (wide_path(from), wide_path(to));
    let flags = MOVEFILE_REPLACE_EXISTING;
    posix_rename(from, &wide_to)
        || unsafe { MoveFileExW(wide_from.as_ptr(), wide_to.as_ptr(), flags) != 0 }
}

fn posix_rename(from: &str, wide_to: &[u16]) -> bool {
    let flags = FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS;
    let source = open_existing(from, DELETE, flags);
    if source == INVALID {
        return false;
    }
    let info = rename_info(wide_to);
    let size = (info.len() * core::mem::size_of::<u64>()) as u32;
    let class = FILE_RENAME_INFO_EX;
    let moved =
        unsafe { SetFileInformationByHandle(source, class, info.as_ptr().cast(), size) } != 0;
    unsafe { CloseHandle(source) };
    moved
}

fn rename_info(wide_to: &[u16]) -> Vec<u64> {
    let name = &wide_to[..wide_to.len() - 1];
    let start = core::mem::offset_of!(RenameInfo, name);
    let bytes = core::mem::size_of_val(wide_to);
    let size = core::cmp::max(core::mem::size_of::<RenameInfo>(), start + bytes);
    let mut info = vec![0u64; size.div_ceil(core::mem::size_of::<u64>())];
    let header = RenameInfo {
        flags: FILE_RENAME_FLAG_REPLACE_IF_EXISTS | FILE_RENAME_FLAG_POSIX_SEMANTICS,
        root: 0,
        name_bytes: core::mem::size_of_val(name) as u32,
        name: [0],
    };
    unsafe {
        info.as_mut_ptr().cast::<RenameInfo>().write(header);
        let into = info.as_mut_ptr().cast::<u8>().add(start).cast::<u16>();
        core::ptr::copy_nonoverlapping(name.as_ptr(), into, name.len());
    }
    info
}
