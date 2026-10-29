use super::types::{FindData, Handle, HandleInformation, MemoryStatus, Overlapped};
use core::ffi::c_void;

#[link(name = "kernel32", kind = "raw-dylib")]
extern "system" {
    pub fn GetStdHandle(which: u32) -> Handle;
    pub fn GetFileType(file: Handle) -> u32;
    pub fn GetConsoleMode(console: Handle, mode: *mut u32) -> i32;
    pub fn SetConsoleMode(console: Handle, mode: u32) -> i32;
    pub fn WriteConsoleW(
        console: Handle,
        text: *const u16,
        len: u32,
        written: *mut u32,
        reserved: *mut c_void,
    ) -> i32;
    pub fn ReadFile(
        file: Handle,
        buf: *mut u8,
        len: u32,
        read: *mut u32,
        overlapped: *mut c_void,
    ) -> i32;
    pub fn WriteFile(
        file: Handle,
        buf: *const u8,
        len: u32,
        written: *mut u32,
        overlapped: *mut c_void,
    ) -> i32;
    pub fn CreateFileW(
        name: *const u16,
        access: u32,
        share: u32,
        security: *mut c_void,
        disposition: u32,
        flags: u32,
        template: Handle,
    ) -> Handle;
    pub fn CloseHandle(handle: Handle) -> i32;
    pub fn GetFileSizeEx(file: Handle, size: *mut i64) -> i32;
    pub fn GetFileInformationByHandle(file: Handle, info: *mut HandleInformation) -> i32;
    pub fn GetFileAttributesW(name: *const u16) -> u32;
    pub fn GetDriveTypeW(root: *const u16) -> u32;
    pub fn GetFinalPathNameByHandleW(file: Handle, buf: *mut u16, size: u32, flags: u32) -> u32;
    pub fn MoveFileExW(from: *const u16, to: *const u16, flags: u32) -> i32;
    pub fn DeleteFileW(name: *const u16) -> i32;
    pub fn CreateHardLinkW(link: *const u16, existing: *const u16, security: *mut c_void) -> i32;
    pub fn SetFileInformationByHandle(
        file: Handle,
        class: u32,
        info: *const c_void,
        size: u32,
    ) -> i32;
    pub fn CreateEventW(
        security: *mut c_void,
        manual: i32,
        initial: i32,
        name: *const u16,
    ) -> Handle;
    pub fn LockFileEx(
        file: Handle,
        flags: u32,
        reserved: u32,
        low: u32,
        high: u32,
        overlapped: *mut Overlapped,
    ) -> i32;
    pub fn UnlockFile(file: Handle, offset: u32, offset_high: u32, low: u32, high: u32) -> i32;
    pub fn GetOverlappedResultEx(
        file: Handle,
        overlapped: *mut Overlapped,
        moved: *mut u32,
        ms: u32,
        alertable: i32,
    ) -> i32;
    pub fn CancelIoEx(file: Handle, overlapped: *mut Overlapped) -> i32;
    pub fn GetLastError() -> u32;
    pub fn CreateDirectoryW(name: *const u16, security: *mut c_void) -> i32;
    pub fn FindFirstFileW(pattern: *const u16, data: *mut FindData) -> Handle;
    pub fn FindNextFileW(search: Handle, data: *mut FindData) -> i32;
    pub fn FindClose(search: Handle) -> i32;
    pub fn GetCommandLineW() -> *const u16;
    pub fn GetEnvironmentVariableW(name: *const u16, buf: *mut u16, size: u32) -> u32;
    pub fn GetModuleFileNameW(module: Handle, buf: *mut u16, size: u32) -> u32;
    pub fn GetSystemDirectoryW(buf: *mut u16, size: u32) -> u32;
    pub fn GetUserDefaultUILanguage() -> u16;
    pub fn GetCurrentDirectoryW(size: u32, buf: *mut u16) -> u32;
    pub fn GetFullPathNameW(name: *const u16, size: u32, buf: *mut u16, file: *mut *mut u16)
        -> u32;
    pub fn GetCurrentProcessId() -> u32;
    pub fn GetSystemTimePreciseAsFileTime(time: *mut u64);
    pub fn QueryPerformanceCounter(count: *mut i64) -> i32;
    pub fn QueryPerformanceFrequency(frequency: *mut i64) -> i32;
    pub fn GetSystemTimes(idle: *mut u64, kernel: *mut u64, user: *mut u64) -> i32;
    pub fn GlobalMemoryStatusEx(status: *mut MemoryStatus) -> i32;
    pub fn DeviceIoControl(
        device: Handle,
        code: u32,
        input: *mut c_void,
        input_len: u32,
        output: *mut c_void,
        output_len: u32,
        returned: *mut u32,
        overlapped: *mut c_void,
    ) -> i32;
    pub fn RegGetValueW(
        key: isize,
        subkey: *const u16,
        value: *const u16,
        flags: u32,
        kind: *mut u32,
        data: *mut c_void,
        size: *mut u32,
    ) -> i32;
}
