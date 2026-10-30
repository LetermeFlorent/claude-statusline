use super::ffi::*;
use super::host;
use std::ffi::c_void;

const WIRE_COUNT_OFFSET: usize = 12;
const PURGEABLE_COUNT_OFFSET: usize = 88;
const COMPRESSOR_PAGE_COUNT_OFFSET: usize = 128;
const INTERNAL_PAGE_COUNT_OFFSET: usize = 140;

pub fn memory() -> Option<(f64, f64)> {
    let mut total = 0u64;
    let mut len = 8usize;
    let r = unsafe {
        sysctlbyname(
            c"hw.memsize".as_ptr().cast(),
            &mut total as *mut u64 as *mut c_void,
            &mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    if r != 0 || total == 0 {
        return None;
    }
    let host = host();
    let mut page = 0usize;
    if unsafe { host_page_size(host, &mut page) } != 0 {
        return None;
    }
    let mut vm_stats = [0i32; 38];
    let mut n = 38u32;
    if unsafe { host_statistics64(host, 4, vm_stats.as_mut_ptr(), &mut n) } != 0 {
        return None;
    }
    let at = |off: usize| vm_stats[off / 4] as u32 as f64;
    let app_pages = at(INTERNAL_PAGE_COUNT_OFFSET) - at(PURGEABLE_COUNT_OFFSET);
    let pages = app_pages + at(WIRE_COUNT_OFFSET) + at(COMPRESSOR_PAGE_COUNT_OFFSET);
    let used = pages * page as f64;
    let used = if used > total as f64 {
        total as f64
    } else {
        used
    };
    Some((total as f64, total as f64 - used))
}
