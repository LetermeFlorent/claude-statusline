use crate::os::windows::files::{read_all, write_handle};
use crate::os::windows::sys::*;
use alloc::string::String;
use core::ptr::null_mut;

const CONSOLE_CHUNK_UNITS: usize = 1024;
const STDIN_SIZE_HINT: usize = 4096;

pub fn stdin() -> Option<String> {
    let input = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
    if input == 0 || input == INVALID || unsafe { GetFileType(input) } == FILE_TYPE_CHAR {
        return None;
    }
    let data = read_all(input, STDIN_SIZE_HINT, usize::MAX);
    Some(String::from_utf8_lossy(&data).into_owned())
}

fn write_console_units(console: Handle, units: &[u16]) -> bool {
    let mut done = 0usize;
    while done < units.len() {
        let mut written = 0u32;
        let len = (units.len() - done) as u32;
        let text = unsafe { units.as_ptr().add(done) };
        if unsafe { WriteConsoleW(console, text, len, &mut written, null_mut()) } == 0
            || written == 0
        {
            return false;
        }
        done += written as usize;
    }
    true
}

pub fn in_utf16_chunks(s: &str, buffer: &mut [u16], write: &mut dyn FnMut(&[u16]) -> bool) -> bool {
    let mut filled = 0usize;
    for c in s.chars() {
        if filled + c.len_utf16() > buffer.len() {
            if !write(&buffer[..filled]) {
                return false;
            }
            filled = 0;
        }
        filled += c.encode_utf16(&mut buffer[filled..]).len();
    }
    filled == 0 || write(&buffer[..filled])
}

fn write_console(console: Handle, s: &str) {
    let mut units = [0u16; CONSOLE_CHUNK_UNITS];
    in_utf16_chunks(s, &mut units, &mut |chunk| {
        write_console_units(console, chunk)
    });
}

fn put(which: u32, s: &str) {
    let output = unsafe { GetStdHandle(which) };
    if output == 0 || output == INVALID {
        return;
    }
    let mut mode = 0u32;
    if unsafe { GetConsoleMode(output, &mut mode) } != 0 {
        unsafe { SetConsoleMode(output, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) };
        write_console(output, s);
    } else {
        write_handle(output, s.as_bytes());
    }
}

pub fn out(s: &str) {
    put(STD_OUTPUT_HANDLE, s);
}

pub fn err(s: &str) {
    put(STD_ERROR_HANDLE, s);
}
