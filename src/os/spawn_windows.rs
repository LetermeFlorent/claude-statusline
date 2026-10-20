use super::windows::wide;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::ptr::{null, null_mut};

type Handle = isize;

#[link(name = "kernel32", kind = "raw-dylib")]
extern "system" {
    fn CreateProcessW(app: *const u16, cmd: *mut u16, pa: *mut c_void, ta: *mut c_void, inherit: i32, flags: u32, env: *mut c_void, dir: *const u16, si: *mut u64, pi: *mut Handle) -> i32;
    fn WaitForSingleObject(h: Handle, ms: u32) -> u32;
    fn GetExitCodeProcess(h: Handle, code: *mut u32) -> i32;
    fn TerminateProcess(h: Handle, code: u32) -> i32;
    fn CloseHandle(h: Handle) -> i32;
    fn Sleep(ms: u32);
}

const NO_WINDOW: u32 = 0x0800_0000;
const BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;
const BELOW_NORMAL_PRIORITY: u32 = 0x4000;
const STARTUP_INFO_SIZE: u64 = 104;
const SIGNALED: u32 = 0;

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
    for a in args {
        line.push(' ');
        quote(a, &mut line);
    }
    wide(&line)
}

fn start(exe: &str, args: &[&str], flags: u32) -> Option<[Handle; 3]> {
    let app = if exe.contains(['\\', '/']) { Some(wide(exe)) } else { None };
    let mut cmd = command_line(exe, args);
    let mut si = [0u64; 13];
    si[0] = STARTUP_INFO_SIZE;
    let mut pi: [Handle; 3] = [0; 3];
    let app_ptr = app.as_ref().map_or(null(), |a| a.as_ptr());
    let ok = unsafe { CreateProcessW(app_ptr, cmd.as_mut_ptr(), null_mut(), null_mut(), 0, flags, null_mut(), null(), si.as_mut_ptr(), pi.as_mut_ptr()) };
    if ok == 0 {
        None
    } else {
        Some(pi)
    }
}

fn close(pi: [Handle; 3]) {
    unsafe {
        CloseHandle(pi[1]);
        CloseHandle(pi[0]);
    }
}

pub fn spawn(exe: &str, args: &[&str]) -> bool {
    let flags = NO_WINDOW | BELOW_NORMAL_PRIORITY;
    match start(exe, args, flags | BREAKAWAY_FROM_JOB).or_else(|| start(exe, args, flags)) {
        Some(pi) => {
            close(pi);
            true
        }
        None => false,
    }
}

pub fn call(exe: &str, args: &[&str], secs: u32) -> Option<i32> {
    let pi = start(exe, args, NO_WINDOW)?;
    let mut code = 0u32;
    let done = unsafe { WaitForSingleObject(pi[0], secs.saturating_mul(1000)) } == SIGNALED;
    if done {
        unsafe { GetExitCodeProcess(pi[0], &mut code) };
    } else {
        unsafe { TerminateProcess(pi[0], 1) };
    }
    close(pi);
    if done {
        Some(code as i32)
    } else {
        None
    }
}

pub fn sleep(ms: u32) {
    unsafe { Sleep(ms) }
}
