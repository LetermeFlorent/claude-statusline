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
}

const NO_WINDOW: u32 = 0x0800_0000;
const BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;
const BELOW_NORMAL_PRIORITY: u32 = 0x4000;
const SIGNALED: u32 = 0;
const TIMED_OUT_EXIT_CODE: u32 = 1;
const MS_PER_SECOND: u32 = 1000;

#[repr(C)]
struct StartupInfo {
    size: u32,
    zeroed: [u64; 12],
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

fn start(exe: &str, args: &[&str], dir: &str, flags: u32) -> Option<ProcessInfo> {
    let app = wide(exe);
    let folder = wide(dir);
    let mut command = command_line(exe, args);
    let mut startup = StartupInfo {
        size: size_of::<StartupInfo>() as u32,
        zeroed: [0; 12],
    };
    let mut child = ProcessInfo::default();
    let ok = unsafe {
        CreateProcessW(
            app.as_ptr(),
            command.as_mut_ptr(),
            null_mut(),
            null_mut(),
            0,
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
    let detached = start(exe, args, dir, flags | BREAKAWAY_FROM_JOB);
    match detached.or_else(|| start(exe, args, dir, flags)) {
        Some(child) => {
            close(child);
            true
        }
        None => false,
    }
}

pub fn call(exe: &str, args: &[&str], dir: &str, secs: u32) -> Option<i32> {
    let child = start(exe, args, dir, NO_WINDOW)?;
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

pub fn sleep(ms: u32) {
    unsafe { Sleep(ms) }
}
