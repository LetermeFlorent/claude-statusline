use crate::os::windows::paths::wide;
use crate::os::windows::sys::*;
use alloc::string::String;
use core::ffi::c_void;
use core::ptr::null_mut;

const LIGHT_THEME_BY_DEFAULT: bool = true;

pub fn os_theme() -> Option<bool> {
    let key = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
    let name = wide("AppsUseLightTheme");
    let mut light = 0u32;
    let mut size = size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_DWORD,
            null_mut(),
            &mut light as *mut u32 as *mut c_void,
            &mut size,
        )
    };
    match status {
        ERROR_SUCCESS => Some(light != 0),
        ERROR_FILE_NOT_FOUND => Some(LIGHT_THEME_BY_DEFAULT),
        _ => None,
    }
}

pub fn os_stamp() -> String {
    String::from(match os_theme() {
        Some(true) => "w1",
        Some(false) => "w0",
        None => "w-",
    })
}
