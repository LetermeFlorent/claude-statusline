use super::child_sys::*;
use super::cmdline::command_line;
use crate::os::windows::paths::wide;
use crate::os::windows::sys::*;
use core::ptr::null_mut;

const NO_WINDOW: u32 = 0x0800_0000;
const BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;
const BELOW_NORMAL_PRIORITY: u32 = 0x4000;
const SIGNALED: u32 = 0;
const TIMED_OUT_EXIT_CODE: u32 = 1;
const MS_PER_SECOND: u32 = 1000;
const NO_HANDLE: Handle = 0;
const USE_STD_HANDLES: u32 = 0x100;

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
