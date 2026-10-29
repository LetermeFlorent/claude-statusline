use super::windows::wide;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::ptr::null_mut;

type Handle = isize;

#[link(name = "kernel32", kind = "raw-dylib")]
extern "system" {
    fn CreateProcessW(
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
    fn WaitForSingleObject(handle: Handle, ms: u32) -> u32;
    fn GetExitCodeProcess(process: Handle, code: *mut u32) -> i32;
    fn TerminateProcess(process: Handle, code: u32) -> i32;
    fn CloseHandle(handle: Handle) -> i32;
    fn Sleep(ms: u32);
    fn CreateFileW(
        name: *const u16,
        access: u32,
        share: u32,
        security: *mut c_void,
        disposition: u32,
        flags: u32,
        template: Handle,
    ) -> Handle;
}

const NO_WINDOW: u32 = 0x0800_0000;
const BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;
const BELOW_NORMAL_PRIORITY: u32 = 0x4000;
const SIGNALED: u32 = 0;
const TIMED_OUT_EXIT_CODE: u32 = 1;
const MS_PER_SECOND: u32 = 1000;
const NO_HANDLE: Handle = 0;
const INVALID: Handle = -1;
const USE_STD_HANDLES: u32 = 0x100;
const GENERIC_WRITE: u32 = 0x4000_0000;
const FILE_SHARE_READ: u32 = 1;
const CREATE_ALWAYS: u32 = 2;
const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;

#[repr(C)]
struct SecurityAttributes {
    size: u32,
    descriptor: *mut c_void,
    inherit: i32,
}

#[repr(C)]
#[derive(Default)]
struct StartupInfo {
    size: u32,
    texts: [usize; 3],
    placement: [u32; 7],
    flags: u32,
    show: [u16; 2],
    reserved: usize,
    handles: [Handle; 3],
}

#[repr(C)]
#[derive(Default)]
struct ProcessInfo {
    process: Handle,
    thread: Handle,
    process_id: u32,
    thread_id: u32,
}

fn quote(arg: &str, out: &mut String) {
    if !arg.is_empty() && !arg.contains([' ', '\t', '"']) {
        out.push_str(arg);
        return;
    }
    out.push('"');
    let mut slashes = 0;
    for c in arg.chars() {
        if c == '\\' {
            slashes += 1;
            continue;
        }
        let n = if c == '"' { slashes * 2 + 1 } else { slashes };
        out.extend(core::iter::repeat_n('\\', n));
        slashes = 0;
        out.push(c);
    }
    out.extend(core::iter::repeat_n('\\', slashes * 2));
    out.push('"');
}

fn command_line(exe: &str, args: &[&str]) -> Vec<u16> {
    let mut line = String::new();
    quote(exe, &mut line);
    for arg in args {
        line.push(' ');
        quote(arg, &mut line);
    }
    wide(&line)
}

fn start(exe: &str, args: &[&str], dir: &str, flags: u32, output: Handle) -> Option<ProcessInfo> {
    let app = wide(exe);
    let folder = wide(dir);
    let mut command = command_line(exe, args);
    let mut startup = StartupInfo {
        size: size_of::<StartupInfo>() as u32,
        ..StartupInfo::default()
    };
    let inherit = output != NO_HANDLE;
    if inherit {
        startup.flags = USE_STD_HANDLES;
        startup.handles = [NO_HANDLE, output, NO_HANDLE];
    }
    let mut child = ProcessInfo::default();
    let ok = unsafe {
        CreateProcessW(
            app.as_ptr(),
            command.as_mut_ptr(),
            null_mut(),
            null_mut(),
            i32::from(inherit),
            flags,
            null_mut(),
            folder.as_ptr(),
            &mut startup,
            &mut child,
        )
    };
    if ok == 0 {
        None
    } else {
        Some(child)
    }
}

fn close(child: ProcessInfo) {
    unsafe {
        CloseHandle(child.thread);
        CloseHandle(child.process);
    }
}

pub fn spawn(exe: &str, args: &[&str], dir: &str) -> bool {
    let flags = NO_WINDOW | BELOW_NORMAL_PRIORITY;
    let detached = start(exe, args, dir, flags | BREAKAWAY_FROM_JOB, NO_HANDLE);
    match detached.or_else(|| start(exe, args, dir, flags, NO_HANDLE)) {
        Some(child) => {
            close(child);
            true
        }
        None => false,
    }
}

fn finish(child: ProcessInfo, secs: u32) -> Option<i32> {
    let mut code = 0u32;
    let timeout = secs.saturating_mul(MS_PER_SECOND);
    let done = unsafe { WaitForSingleObject(child.process, timeout) } == SIGNALED;
    if done {
        unsafe { GetExitCodeProcess(child.process, &mut code) };
    } else {
        unsafe { TerminateProcess(child.process, TIMED_OUT_EXIT_CODE) };
    }
    close(child);
    if done {
        Some(code as i32)
    } else {
        None
    }
}

pub fn call(exe: &str, args: &[&str], dir: &str, secs: u32) -> Option<i32> {
    finish(start(exe, args, dir, NO_WINDOW, NO_HANDLE)?, secs)
}

pub fn call_to(exe: &str, args: &[&str], dir: &str, secs: u32, output: &str) -> Option<i32> {
    let mut security = SecurityAttributes {
        size: size_of::<SecurityAttributes>() as u32,
        descriptor: null_mut(),
        inherit: 1,
    };
    let path = wide(output);
    let file = unsafe {
        CreateFileW(
            path.as_ptr(),
            GENERIC_WRITE,
            FILE_SHARE_READ,
            (&mut security as *mut SecurityAttributes).cast(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            NO_HANDLE,
        )
    };
    if file == INVALID {
        return None;
    }
    let code = start(exe, args, dir, NO_WINDOW, file).and_then(|child| finish(child, secs));
    unsafe { CloseHandle(file) };
    code
}

pub fn sleep(ms: u32) {
    unsafe { Sleep(ms) }
}
