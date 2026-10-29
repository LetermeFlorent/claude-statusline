use super::read::open_existing;
use crate::os::windows::paths::wide_path;
use crate::os::windows::sys::*;

pub fn stamp(path: &str) -> Option<(u64, i64)> {
    let file = open_existing(path, FILE_READ_ATTRIBUTES, FILE_FLAG_BACKUP_SEMANTICS);
    if file == INVALID {
        return None;
    }
    let mut info = HandleInformation::default();
    let read = unsafe { GetFileInformationByHandle(file, &mut info) } != 0;
    unsafe { CloseHandle(file) };
    let since_epoch = info.written.ticks() as i64 - UNIX_EPOCH_TICKS as i64;
    let size = (info.size_high as u64) << 32 | info.size_low as u64;
    read.then_some((size, since_epoch.div_euclid(TICKS_PER_SECOND)))
}

pub fn exists(path: &str) -> bool {
    let wide_path = wide_path(path);
    unsafe { GetFileAttributesW(wide_path.as_ptr()) != u32::MAX }
}
