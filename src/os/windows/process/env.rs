use crate::os::join;
use crate::os::windows::paths::{wide, with_growing_buffer};
use crate::os::windows::sys::*;
use alloc::string::String;

pub fn env(name: &str) -> Option<String> {
    let wide_name = wide(name);
    with_growing_buffer(&|b, n| unsafe { GetEnvironmentVariableW(wide_name.as_ptr(), b, n) })
}

pub fn pid() -> u32 {
    unsafe { GetCurrentProcessId() }
}

pub fn exe() -> Option<String> {
    with_growing_buffer(&|b, n| unsafe { GetModuleFileNameW(0, b, n) })
}

pub fn system_dir() -> Option<String> {
    with_growing_buffer(&|b, n| unsafe { GetSystemDirectoryW(b, n) })
}

pub fn ui_language() -> u16 {
    unsafe { GetUserDefaultUILanguage() }
}

pub fn cwd() -> Option<String> {
    with_growing_buffer(&|b, n| unsafe { GetCurrentDirectoryW(n, b) })
}

pub fn home() -> Option<String> {
    env("USERPROFILE")
        .filter(|s| !s.is_empty())
        .or_else(|| env("HOME").filter(|s| !s.is_empty()))
}

pub fn state_dir() -> Option<String> {
    let base = env("LOCALAPPDATA")
        .filter(|s| !s.is_empty())
        .or_else(|| home().map(|h| join(&h, &["AppData", "Local"])))?;
    Some(join(&base, &["claude-statusline"]))
}
