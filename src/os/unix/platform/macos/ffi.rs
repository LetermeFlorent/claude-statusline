use std::ffi::c_void;

pub type Port = u32;

extern "C" {
    pub fn mach_host_self() -> Port;
    pub fn host_statistics(host: Port, flavor: i32, info: *mut i32, count: *mut u32) -> i32;
    pub fn host_statistics64(host: Port, flavor: i32, info: *mut i32, count: *mut u32) -> i32;
    pub fn host_page_size(host: Port, size: *mut usize) -> i32;
    pub fn sysctlbyname(
        name: *const u8,
        old: *mut c_void,
        oldlen: *mut usize,
        new: *mut c_void,
        newlen: usize,
    ) -> i32;
    pub fn clock_gettime_nsec_np(clock: u32) -> u64;
    #[cfg_attr(target_arch = "x86_64", link_name = "getfsstat$INODE64")]
    pub fn getfsstat(entries: *mut u64, size: i32, mode: i32) -> i32;
}

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    pub fn IOServiceMatching(name: *const u8) -> *mut c_void;
    pub fn IOServiceGetMatchingServices(main: Port, matching: *mut c_void, iter: *mut u32) -> i32;
    pub fn IOIteratorNext(iter: u32) -> u32;
    pub fn IORegistryEntryCreateCFProperty(
        entry: u32,
        key: *const c_void,
        alloc: *const c_void,
        options: u32,
    ) -> *const c_void;
    pub fn IOObjectRelease(obj: u32) -> i32;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    pub fn CFStringCreateWithCString(alloc: *const c_void, s: *const u8, enc: u32)
        -> *const c_void;
    pub fn CFDictionaryGetValue(d: *const c_void, key: *const c_void) -> *const c_void;
    pub fn CFNumberGetValue(n: *const c_void, kind: i32, out: *mut c_void) -> u8;
    pub fn CFGetTypeID(cf: *const c_void) -> usize;
    pub fn CFDictionaryGetTypeID() -> usize;
    pub fn CFNumberGetTypeID() -> usize;
    pub fn CFRelease(cf: *const c_void);
}
