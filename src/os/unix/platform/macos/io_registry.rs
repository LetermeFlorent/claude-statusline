use super::ffi::*;
use crate::os::Activity;
use std::ffi::c_void;

fn cfstr(s: &[u8]) -> *const c_void {
    unsafe { CFStringCreateWithCString(std::ptr::null(), s.as_ptr(), 0x0800_0100) }
}

fn number(d: *const c_void, key: *const c_void) -> i64 {
    let v = unsafe { CFDictionaryGetValue(d, key) };
    let mut out = 0i64;
    if !v.is_null() && unsafe { CFGetTypeID(v) == CFNumberGetTypeID() } {
        unsafe { CFNumberGetValue(v, 4, &mut out as *mut i64 as *mut c_void) };
    }
    out
}

const CLOCK_UPTIME_RAW: u32 = 8;

pub fn disk(_path: &str) -> Option<Activity> {
    let mut iter = 0u32;
    let matching = unsafe { IOServiceMatching(c"IOBlockStorageDriver".as_ptr().cast()) };
    if matching.is_null() || unsafe { IOServiceGetMatchingServices(0, matching, &mut iter) } != 0 {
        return None;
    }
    let k_stats = cfstr(b"Statistics\0");
    let k_read = cfstr(b"Total Time (Read)\0");
    let k_write = cfstr(b"Total Time (Write)\0");
    let mut busy_ns = 0i64;
    let mut drivers = 0u64;
    loop {
        let entry = unsafe { IOIteratorNext(iter) };
        if entry == 0 {
            break;
        }
        let stats = unsafe { IORegistryEntryCreateCFProperty(entry, k_stats, std::ptr::null(), 0) };
        if !stats.is_null() {
            if unsafe { CFGetTypeID(stats) == CFDictionaryGetTypeID() } {
                busy_ns = busy_ns
                    .wrapping_add(number(stats, k_read))
                    .wrapping_add(number(stats, k_write));
                drivers += 1;
            }
            unsafe { CFRelease(stats) };
        }
        unsafe { IOObjectRelease(entry) };
    }
    unsafe {
        IOObjectRelease(iter);
        CFRelease(k_stats);
        CFRelease(k_read);
        CFRelease(k_write);
    }
    if drivers == 0 {
        return None;
    }
    let uptime_ns = unsafe { clock_gettime_nsec_np(CLOCK_UPTIME_RAW) } as i64;
    Some(Activity {
        source: drivers,
        clock: uptime_ns,
        busy: busy_ns,
    })
}
