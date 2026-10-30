use super::super::sys::Handle;
use core::ffi::c_void;

#[link(name = "kernel32", kind = "raw-dylib")]
extern "system" {
    pub fn CreateProcessW(
        app: *const u16,
        command: *mut u16,
        process_security: *mut c_void,
        thread_security: *mut c_void,
        inherit_handles: i32,
        flags: u32,
        environment: *mut c_void,
        directory: *const u16,
        startup: *mut StartupInfo,
        child: *mut ProcessInfo,
    ) -> i32;
    pub fn WaitForSingleObject(handle: Handle, ms: u32) -> u32;
    pub fn GetExitCodeProcess(process: Handle, code: *mut u32) -> i32;
    pub fn TerminateProcess(process: Handle, code: u32) -> i32;
    pub fn Sleep(ms: u32);
}

#[repr(C)]
pub struct SecurityAttributes {
    pub size: u32,
    pub descriptor: *mut c_void,
    pub inherit: i32,
}

#[repr(C)]
#[derive(Default)]
pub struct StartupInfo {
    pub size: u32,
    pub texts: [usize; 3],
    pub placement: [u32; 7],
    pub flags: u32,
    pub show: [u16; 2],
    pub reserved: usize,
    pub handles: [Handle; 3],
}

#[repr(C)]
#[derive(Default)]
pub struct ProcessInfo {
    pub process: Handle,
    pub thread: Handle,
    pub process_id: u32,
    pub thread_id: u32,
}
