use super::{Activity, Claim, CPU_SOURCE};
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::ptr::null_mut;

pub const SEP: char = '\\';

type Handle = isize;
const INVALID: Handle = -1;

#[link(name = "kernel32", kind = "raw-dylib")]
extern "system" {
    fn GetStdHandle(which: u32) -> Handle;
    fn GetFileType(file: Handle) -> u32;
    fn GetConsoleMode(console: Handle, mode: *mut u32) -> i32;
    fn SetConsoleMode(console: Handle, mode: u32) -> i32;
    fn WriteConsoleW(
        console: Handle,
        text: *const u16,
        len: u32,
        written: *mut u32,
        reserved: *mut c_void,
    ) -> i32;
    fn ReadFile(
        file: Handle,
        buf: *mut u8,
        len: u32,
        read: *mut u32,
        overlapped: *mut c_void,
    ) -> i32;
    fn WriteFile(
        file: Handle,
        buf: *const u8,
        len: u32,
        written: *mut u32,
        overlapped: *mut c_void,
    ) -> i32;
    fn CreateFileW(
        name: *const u16,
        access: u32,
        share: u32,
        security: *mut c_void,
        disposition: u32,
        flags: u32,
        template: Handle,
    ) -> Handle;
    fn CloseHandle(handle: Handle) -> i32;
    fn GetFileSizeEx(file: Handle, size: *mut i64) -> i32;
    fn GetFileInformationByHandle(file: Handle, info: *mut HandleInformation) -> i32;
    fn GetFileAttributesW(name: *const u16) -> u32;
    fn GetDriveTypeW(root: *const u16) -> u32;
    fn GetFinalPathNameByHandleW(file: Handle, buf: *mut u16, size: u32, flags: u32) -> u32;
    fn MoveFileExW(from: *const u16, to: *const u16, flags: u32) -> i32;
    fn DeleteFileW(name: *const u16) -> i32;
    fn CreateHardLinkW(link: *const u16, existing: *const u16, security: *mut c_void) -> i32;
    fn SetFileInformationByHandle(file: Handle, class: u32, info: *const c_void, size: u32) -> i32;
    fn CreateEventW(security: *mut c_void, manual: i32, initial: i32, name: *const u16) -> Handle;
    fn LockFileEx(
        file: Handle,
        flags: u32,
        reserved: u32,
        low: u32,
        high: u32,
        overlapped: *mut Overlapped,
    ) -> i32;
    fn UnlockFile(file: Handle, offset: u32, offset_high: u32, low: u32, high: u32) -> i32;
    fn GetOverlappedResultEx(
        file: Handle,
        overlapped: *mut Overlapped,
        moved: *mut u32,
        ms: u32,
        alertable: i32,
    ) -> i32;
    fn CancelIoEx(file: Handle, overlapped: *mut Overlapped) -> i32;
    fn GetLastError() -> u32;
    fn CreateDirectoryW(name: *const u16, security: *mut c_void) -> i32;
    fn FindFirstFileW(pattern: *const u16, data: *mut FindData) -> Handle;
    fn FindNextFileW(search: Handle, data: *mut FindData) -> i32;
    fn FindClose(search: Handle) -> i32;
    fn GetCommandLineW() -> *const u16;
    fn GetEnvironmentVariableW(name: *const u16, buf: *mut u16, size: u32) -> u32;
    fn GetModuleFileNameW(module: Handle, buf: *mut u16, size: u32) -> u32;
    fn GetSystemDirectoryW(buf: *mut u16, size: u32) -> u32;
    fn GetUserDefaultUILanguage() -> u16;
    fn GetCurrentDirectoryW(size: u32, buf: *mut u16) -> u32;
    fn GetFullPathNameW(name: *const u16, size: u32, buf: *mut u16, file: *mut *mut u16) -> u32;
    fn GetCurrentProcessId() -> u32;
    fn GetSystemTimePreciseAsFileTime(time: *mut u64);
    fn QueryPerformanceCounter(count: *mut i64) -> i32;
    fn QueryPerformanceFrequency(frequency: *mut i64) -> i32;
    fn GetSystemTimes(idle: *mut u64, kernel: *mut u64, user: *mut u64) -> i32;
    fn GlobalMemoryStatusEx(status: *mut MemoryStatus) -> i32;
    fn DeviceIoControl(
        device: Handle,
        code: u32,
        input: *mut c_void,
        input_len: u32,
        output: *mut c_void,
        output_len: u32,
        returned: *mut u32,
        overlapped: *mut c_void,
    ) -> i32;
    fn RegGetValueW(
        key: isize,
        subkey: *const u16,
        value: *const u16,
        flags: u32,
        kind: *mut u32,
        data: *mut c_void,
        size: *mut u32,
    ) -> i32;
}

const GENERIC_READ: u32 = 0x8000_0000;
const GENERIC_WRITE: u32 = 0x4000_0000;
const FILE_READ_ATTRIBUTES: u32 = 0x80;
const DEVICE_ATTRIBUTES_ONLY: u32 = 0;
const FILE_SHARE_READ: u32 = 1;
const FILE_SHARE_WRITE: u32 = 2;
const FILE_SHARE_DELETE: u32 = 4;
const SHARE_ALL: u32 = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
const SHARE_NONE: u32 = 0;
const OPEN_EXISTING: u32 = 3;
const CREATE_ALWAYS: u32 = 2;
const CREATE_NEW: u32 = 1;
const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
const FILE_TYPE_DISK: u32 = 1;
const FILE_TYPE_CHAR: u32 = 2;
const MOVEFILE_REPLACE_EXISTING: u32 = 1;
const DELETE: u32 = 0x0001_0000;
const OPEN_ALWAYS: u32 = 4;
const FILE_FLAG_OVERLAPPED: u32 = 0x4000_0000;
const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
const FILE_RENAME_INFO_EX: u32 = 22;
const FILE_RENAME_FLAG_REPLACE_IF_EXISTS: u32 = 1;
const FILE_RENAME_FLAG_POSIX_SEMANTICS: u32 = 2;
const LOCKFILE_EXCLUSIVE_LOCK: u32 = 2;
const LOCKED_BYTES: u32 = 1;
const ERROR_IO_PENDING: u32 = 997;
const INFINITE: u32 = u32::MAX;
const MANUAL_RESET: i32 = 1;
const RENAME_TRIES: u32 = 3;
const RENAME_RETRY_MS: u32 = 5;
const STD_INPUT_HANDLE: u32 = -10i32 as u32;
const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;
const STD_ERROR_HANDLE: u32 = -12i32 as u32;
const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 4;
const IOCTL_DISK_PERFORMANCE: u32 = 0x70020;
const HKEY_CURRENT_USER: isize = 0x8000_0001u32 as i32 as isize;
const RRF_RT_REG_DWORD: u32 = 0x10;
const ERROR_SUCCESS: i32 = 0;
const VOLUME_NAME_DOS: u32 = 0;
const ERROR_FILE_NOT_FOUND: i32 = 2;
const MAX_PATH: u32 = 260;
const LEGACY_PATH_UNITS: usize = 248;
const VERBATIM_PREFIX: &str = "\\\\?\\";
const VERBATIM_UNC_PREFIX: &str = "\\\\?\\UNC\\";
const NT_PREFIX: &str = "\\??\\";
const UNC_PREFIX: &str = "\\\\";
const VERBATIM_UNC_NAME: &str = "UNC\\";
const DRIVE_ROOT_INDEX: usize = 2;
const DRIVE_REMOTE: u32 = 4;
const MAX_BUFFER_UNITS: u32 = 1 << 20;
const READ_GROWTH_BYTES: usize = 4 << 10;
const MAX_WRITE_BYTES: usize = 1 << 30;
const CONSOLE_CHUNK_UNITS: usize = 1024;
const STDIN_SIZE_HINT: usize = 4096;
const UNIX_EPOCH_TICKS: u64 = 116_444_736_000_000_000;
const TICKS_PER_SECOND: i64 = 10_000_000;

#[repr(C)]
#[derive(Default)]
struct Overlapped {
    internal: usize,
    internal_high: usize,
    offset: u32,
    offset_high: u32,
    event: Handle,
}

#[repr(C)]
struct RenameInfo {
    flags: u32,
    root: Handle,
    name_bytes: u32,
    name: [u16; 1],
}

#[repr(C)]
#[derive(Default)]
struct FileTime {
    low: u32,
    high: u32,
}

impl FileTime {
    fn ticks(&self) -> u64 {
        (self.high as u64) << 32 | self.low as u64
    }
}

#[repr(C)]
#[derive(Default)]
struct FileAttributeData {
    attributes: u32,
    created: FileTime,
    accessed: FileTime,
    written: FileTime,
    size_high: u32,
    size_low: u32,
}

#[repr(C)]
#[derive(Default)]
struct HandleInformation {
    attributes: u32,
    created: FileTime,
    accessed: FileTime,
    written: FileTime,
    volume: u32,
    size_high: u32,
    size_low: u32,
    links: u32,
    index_high: u32,
    index_low: u32,
}

#[repr(C)]
struct FindData {
    info: FileAttributeData,
    reserved: [u32; 2],
    name: [u16; MAX_PATH as usize],
    short_name: [u16; 14],
}

impl FindData {
    fn new() -> Self {
        FindData {
            info: FileAttributeData::default(),
            reserved: [0; 2],
            name: [0; MAX_PATH as usize],
            short_name: [0; 14],
        }
    }

    fn name(&self) -> String {
        let len = self.name.iter().position(|&c| c == 0);
        String::from_utf16_lossy(&self.name[..len.unwrap_or(self.name.len())])
    }
}

#[repr(C)]
#[derive(Default)]
struct DiskPerformance {
    bytes_read: i64,
    bytes_written: i64,
    read_time: i64,
    write_time: i64,
    idle_time: i64,
    counts: [u32; 4],
    query_time: i64,
    device_number: u32,
    manager_name: [u16; 8],
}

#[repr(C)]
#[derive(Default)]
struct MemoryStatus {
    length: u32,
    load: u32,
    total: u64,
    available: u64,
    page_file: [u64; 2],
    virtual_space: [u64; 2],
    extended_virtual: u64,
}

pub(super) fn wide(s: &str) -> Vec<u16> {
    let mut v: Vec<u16> = s.encode_utf16().collect();
    v.push(0);
    v
}

fn verbatim(full: String) -> String {
    match full.strip_prefix(UNC_PREFIX) {
        Some(rest) if rest.starts_with(['?', '.']) => full,
        Some(rest) => format!("{}{}", VERBATIM_UNC_PREFIX, rest),
        None => format!("{}{}", VERBATIM_PREFIX, full),
    }
}

fn wide_path(path: &str) -> Vec<u16> {
    let plain = wide(path);
    let prefixed = path.starts_with(VERBATIM_PREFIX) || path.starts_with(NT_PREFIX);
    if prefixed || plain.len() < LEGACY_PATH_UNITS {
        return plain;
    }
    let full =
        with_growing_buffer(&|b, n| unsafe { GetFullPathNameW(plain.as_ptr(), n, b, null_mut()) });
    full.map_or(plain, |full| wide(&verbatim(full)))
}

unsafe fn units_until_nul<'a>(text: *const u16) -> &'a [u16] {
    let mut len = 0;
    while *text.add(len) != 0 {
        len += 1;
    }
    core::slice::from_raw_parts(text, len)
}

fn with_growing_buffer(call: &dyn Fn(*mut u16, u32) -> u32) -> Option<String> {
    let mut size = MAX_PATH;
    loop {
        let mut buf: Vec<u16> = vec![0; size as usize];
        let needed = call(buf.as_mut_ptr(), size);
        if needed == 0 {
            return None;
        }
        if needed < size {
            return Some(String::from_utf16_lossy(&buf[..needed as usize]));
        }
        size = if needed > size { needed + 1 } else { size * 2 };
        if size > MAX_BUFFER_UNITS {
            return None;
        }
    }
}

pub fn args() -> Vec<String> {
    let line = unsafe { GetCommandLineW() };
    if line.is_null() {
        return Vec::new();
    }
    split_command_line(unsafe { units_until_nul(line) })
}

const QUOTE: u16 = b'"' as u16;
const BACKSLASH: u16 = b'\\' as u16;
const SPACE: u16 = b' ' as u16;
const TAB: u16 = b'\t' as u16;

fn is_blank(unit: u16) -> bool {
    unit == SPACE || unit == TAB
}

fn program_end(line: &[u16]) -> usize {
    let mut quoted = false;
    line.iter()
        .position(|&unit| {
            quoted ^= unit == QUOTE;
            is_blank(unit) && !quoted
        })
        .unwrap_or(line.len())
}

#[derive(Default)]
struct Words {
    found: Vec<String>,
    word: Vec<u16>,
    started: bool,
    quoted: bool,
}

impl Words {
    fn end_word(&mut self) {
        if self.started {
            self.found.push(String::from_utf16_lossy(&self.word));
        }
        self.word.clear();
        self.started = false;
    }

    fn read(&mut self, line: &[u16], at: usize) -> usize {
        let unit = line[at];
        if is_blank(unit) && !self.quoted {
            self.end_word();
            return at + 1;
        }
        self.started = true;
        match unit {
            BACKSLASH => self.read_backslashes(line, at),
            QUOTE if self.quoted && line.get(at + 1) == Some(&QUOTE) => {
                self.word.push(QUOTE);
                at + 2
            }
            QUOTE => {
                self.quoted = !self.quoted;
                at + 1
            }
            _ => {
                self.word.push(unit);
                at + 1
            }
        }
    }

    fn read_backslashes(&mut self, line: &[u16], at: usize) -> usize {
        let count = line[at..]
            .iter()
            .take_while(|&&unit| unit == BACKSLASH)
            .count();
        let after = at + count;
        if line.get(after) != Some(&QUOTE) {
            self.word.resize(self.word.len() + count, BACKSLASH);
            return after;
        }
        self.word.resize(self.word.len() + count / 2, BACKSLASH);
        if count % 2 == 0 {
            return after;
        }
        self.word.push(QUOTE);
        after + 1
    }
}

fn split_command_line(line: &[u16]) -> Vec<String> {
    let end = program_end(line);
    let program: Vec<u16> = line[..end]
        .iter()
        .copied()
        .filter(|&unit| unit != QUOTE)
        .collect();
    let mut words = Words::default();
    words.found.push(String::from_utf16_lossy(&program));
    let mut at = end;
    while at < line.len() {
        at = words.read(line, at);
    }
    words.end_word();
    words.found
}

pub fn env(name: &str) -> Option<String> {
    let wide_name = wide(name);
    with_growing_buffer(&|b, n| unsafe { GetEnvironmentVariableW(wide_name.as_ptr(), b, n) })
}

fn open_existing(path: &str, access: u32, flags: u32) -> Handle {
    let wide_path = wide_path(path);
    unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            access,
            SHARE_ALL,
            null_mut(),
            OPEN_EXISTING,
            flags,
            0,
        )
    }
}

fn read_all(file: Handle, hint: usize, limit: usize) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(hint.saturating_add(1));
    while out.len() <= limit {
        if out.len() == out.capacity() {
            out.reserve(READ_GROWTH_BYTES);
        }
        let room = (out.capacity() - out.len())
            .min((limit - out.len()).saturating_add(1))
            .min(u32::MAX as usize);
        let mut got = 0u32;
        let ok = unsafe {
            ReadFile(
                file,
                out.as_mut_ptr().add(out.len()),
                room as u32,
                &mut got,
                null_mut(),
            )
        };
        if ok == 0 || got == 0 {
            break;
        }
        unsafe { out.set_len(out.len() + got as usize) };
    }
    out
}

pub fn read_limited(path: &str, limit: usize) -> Option<Vec<u8>> {
    let file = open_existing(path, GENERIC_READ, FILE_ATTRIBUTE_NORMAL);
    if file == INVALID {
        return None;
    }
    let mut size = 0i64;
    let on_disk = unsafe { GetFileType(file) } == FILE_TYPE_DISK
        && unsafe { GetFileSizeEx(file, &mut size) } != 0
        && (0..=limit as i64).contains(&size);
    let data = on_disk.then(|| read_all(file, size as usize, limit));
    unsafe { CloseHandle(file) };
    data.filter(|d| d.len() <= limit)
}

fn write_handle(file: Handle, data: &[u8]) -> bool {
    let mut done = 0usize;
    while done < data.len() {
        let mut written = 0u32;
        let len = core::cmp::min(data.len() - done, MAX_WRITE_BYTES) as u32;
        let ok = unsafe { WriteFile(file, data.as_ptr().add(done), len, &mut written, null_mut()) };
        if ok == 0 || written == 0 {
            return false;
        }
        done += written as usize;
    }
    true
}

fn write_with(path: &str, share: u32, disposition: u32, data: &[u8]) -> bool {
    let wide_path = wide_path(path);
    let file = unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            GENERIC_WRITE,
            share,
            null_mut(),
            disposition,
            FILE_ATTRIBUTE_NORMAL,
            0,
        )
    };
    if file == INVALID {
        return false;
    }
    let ok = write_handle(file, data);
    unsafe { CloseHandle(file) };
    ok
}

pub fn write(path: &str, data: &[u8]) -> bool {
    write_with(path, FILE_SHARE_READ, CREATE_ALWAYS, data)
}

fn resolved(path: &str) -> String {
    let wide_path = wide_path(path);
    let attributes = unsafe { GetFileAttributesW(wide_path.as_ptr()) };
    let linked = attributes != u32::MAX && attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0;
    let file = if linked {
        open_existing(path, GENERIC_READ, FILE_ATTRIBUTE_NORMAL)
    } else {
        INVALID
    };
    if file == INVALID {
        return String::from(path);
    }
    let target = with_growing_buffer(&|b, n| unsafe {
        GetFinalPathNameByHandleW(file, b, n, VOLUME_NAME_DOS)
    });
    unsafe { CloseHandle(file) };
    target.unwrap_or_else(|| String::from(path))
}

pub fn write_atomic(path: &str, data: &[u8]) -> bool {
    let target = resolved(path);
    let tmp = format!("{}.{}.tmp", target, pid());
    let swapped = write(&tmp, data)
        && (0..RENAME_TRIES).any(|attempt| {
            if attempt > 0 {
                super::sleep(RENAME_RETRY_MS);
            }
            rename(&tmp, &target)
        });
    if !swapped {
        remove(&tmp);
    }
    swapped
}

pub fn pid() -> u32 {
    unsafe { GetCurrentProcessId() }
}

pub fn remove(path: &str) -> bool {
    let wide_path = wide_path(path);
    unsafe { DeleteFileW(wide_path.as_ptr()) != 0 }
}

pub fn rename(from: &str, to: &str) -> bool {
    let (wide_from, wide_to) = (wide_path(from), wide_path(to));
    let flags = MOVEFILE_REPLACE_EXISTING;
    posix_rename(from, &wide_to)
        || unsafe { MoveFileExW(wide_from.as_ptr(), wide_to.as_ptr(), flags) != 0 }
}

fn posix_rename(from: &str, wide_to: &[u16]) -> bool {
    let flags = FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS;
    let source = open_existing(from, DELETE, flags);
    if source == INVALID {
        return false;
    }
    let info = rename_info(wide_to);
    let size = (info.len() * core::mem::size_of::<u64>()) as u32;
    let class = FILE_RENAME_INFO_EX;
    let moved =
        unsafe { SetFileInformationByHandle(source, class, info.as_ptr().cast(), size) } != 0;
    unsafe { CloseHandle(source) };
    moved
}

fn rename_info(wide_to: &[u16]) -> Vec<u64> {
    let name = &wide_to[..wide_to.len() - 1];
    let start = core::mem::offset_of!(RenameInfo, name);
    let bytes = core::mem::size_of_val(wide_to);
    let size = core::cmp::max(core::mem::size_of::<RenameInfo>(), start + bytes);
    let mut info = vec![0u64; size.div_ceil(core::mem::size_of::<u64>())];
    let header = RenameInfo {
        flags: FILE_RENAME_FLAG_REPLACE_IF_EXISTS | FILE_RENAME_FLAG_POSIX_SEMANTICS,
        root: 0,
        name_bytes: core::mem::size_of_val(name) as u32,
        name: [0],
    };
    unsafe {
        info.as_mut_ptr().cast::<RenameInfo>().write(header);
        let into = info.as_mut_ptr().cast::<u8>().add(start).cast::<u16>();
        core::ptr::copy_nonoverlapping(name.as_ptr(), into, name.len());
    }
    info
}

pub fn create_private(path: &str, data: &[u8]) -> bool {
    write_with(path, SHARE_NONE, CREATE_NEW, data)
}

pub struct Lock(Handle);

impl Drop for Lock {
    fn drop(&mut self) {
        unsafe {
            UnlockFile(self.0, 0, 0, LOCKED_BYTES, 0);
            CloseHandle(self.0);
        }
    }
}

pub fn lock(path: &str, wait_ms: u32) -> Claim {
    let wide_path = wide_path(path);
    let file = unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            SHARE_ALL,
            null_mut(),
            OPEN_ALWAYS,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
            0,
        )
    };
    if file == INVALID {
        return Claim::Unavailable;
    }
    let held = Lock(file);
    let event = unsafe { CreateEventW(null_mut(), MANUAL_RESET, 0, core::ptr::null()) };
    if event == 0 {
        return Claim::Unavailable;
    }
    let taken = wait_for_lock(file, event, wait_ms);
    unsafe { CloseHandle(event) };
    match taken {
        Some(true) => Claim::Held(held),
        Some(false) => Claim::Busy,
        None => Claim::Unavailable,
    }
}

fn wait_for_lock(file: Handle, event: Handle, wait_ms: u32) -> Option<bool> {
    let mut overlapped = Overlapped {
        event,
        ..Overlapped::default()
    };
    let mut moved = 0u32;
    unsafe {
        if LockFileEx(
            file,
            LOCKFILE_EXCLUSIVE_LOCK,
            0,
            LOCKED_BYTES,
            0,
            &mut overlapped,
        ) != 0
        {
            return Some(true);
        }
        if GetLastError() != ERROR_IO_PENDING {
            return None;
        }
        if GetOverlappedResultEx(file, &mut overlapped, &mut moved, wait_ms, 0) != 0 {
            return Some(true);
        }
        CancelIoEx(file, &mut overlapped);
        Some(GetOverlappedResultEx(file, &mut overlapped, &mut moved, INFINITE, 0) != 0)
    }
}

pub fn make_executable(path: &str) -> bool {
    exists(path)
}

pub fn hard_link(existing: &str, link: &str) -> bool {
    let (wide_existing, wide_link) = (wide_path(existing), wide_path(link));
    unsafe { CreateHardLinkW(wide_link.as_ptr(), wide_existing.as_ptr(), null_mut()) != 0 }
}

pub fn process_id() -> u32 {
    unsafe { GetCurrentProcessId() }
}

pub fn stamp(path: &str) -> Option<(u64, i64)> {
    let file = open_existing(path, FILE_READ_ATTRIBUTES, FILE_FLAG_BACKUP_SEMANTICS);
    if file == INVALID {
        return None;
    }
    let mut info = HandleInformation::default();
    let read = unsafe { GetFileInformationByHandle(file, &mut info) } != 0;
    unsafe { CloseHandle(file) };
    let since_epoch = info.written.ticks() as i64 - UNIX_EPOCH_TICKS as i64;
    let size = (info.size_high as u64) << 32 | info.size_low as u64;
    read.then_some((size, since_epoch.div_euclid(TICKS_PER_SECOND)))
}

pub fn exists(path: &str) -> bool {
    let wide_path = wide_path(path);
    unsafe { GetFileAttributesW(wide_path.as_ptr()) != u32::MAX }
}

fn is_separator(c: char) -> bool {
    c == '\\' || c == '/'
}

fn remote_drive(path: &str) -> bool {
    let mut start = path.chars();
    match (start.next(), start.next()) {
        (Some(letter), Some(':')) if letter.is_ascii_alphabetic() => {
            let root = wide(&format!("{}:\\", letter));
            unsafe { GetDriveTypeW(root.as_ptr()) == DRIVE_REMOTE }
        }
        _ => false,
    }
}

pub fn on_network(path: &str) -> bool {
    let (rest, unc) = match path.strip_prefix(VERBATIM_PREFIX) {
        Some(rest) => (
            rest,
            rest.get(..VERBATIM_UNC_NAME.len())
                .is_some_and(|name| name.eq_ignore_ascii_case(VERBATIM_UNC_NAME)),
        ),
        None => (
            path,
            path.starts_with(is_separator) && path[1..].starts_with(is_separator),
        ),
    };
    unc || remote_drive(rest)
}

pub fn mkdirs(path: &str) -> bool {
    for (i, c) in path.char_indices() {
        if (c == '\\' || c == '/') && i > DRIVE_ROOT_INDEX {
            let wide_parent = wide_path(&path[..i]);
            unsafe { CreateDirectoryW(wide_parent.as_ptr(), null_mut()) };
        }
    }
    let wide_path = wide_path(path);
    unsafe { CreateDirectoryW(wide_path.as_ptr(), null_mut()) };
    exists(path)
}

pub fn list(dir: &str, prefix: &str) -> Vec<String> {
    let mut out = Vec::new();
    let pattern = wide_path(&format!("{}\\{}*", dir, prefix));
    let mut data = FindData::new();
    let search = unsafe { FindFirstFileW(pattern.as_ptr(), &mut data) };
    if search == INVALID {
        return out;
    }
    loop {
        let name = data.name();
        if name != "." && name != ".." {
            out.push(name);
        }
        if unsafe { FindNextFileW(search, &mut data) } == 0 {
            break;
        }
    }
    unsafe { FindClose(search) };
    out
}

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

fn in_utf16_chunks(s: &str, buffer: &mut [u16], write: &mut dyn FnMut(&[u16]) -> bool) -> bool {
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

pub fn now() -> f64 {
    let mut ticks = 0u64;
    unsafe { GetSystemTimePreciseAsFileTime(&mut ticks) };
    (ticks as f64 - UNIX_EPOCH_TICKS as f64) / TICKS_PER_SECOND as f64
}

pub fn ticks() -> f64 {
    let (mut count, mut frequency) = (0i64, 0i64);
    unsafe {
        QueryPerformanceCounter(&mut count);
        QueryPerformanceFrequency(&mut frequency);
    }
    if frequency == 0 {
        0.0
    } else {
        count as f64 / frequency as f64
    }
}

pub fn cpu() -> Option<Activity> {
    let (mut idle, mut kernel, mut user) = (0u64, 0u64, 0u64);
    if unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) } == 0 {
        return None;
    }
    let clock = kernel.wrapping_add(user);
    Some(Activity {
        source: CPU_SOURCE,
        clock: clock as i64,
        busy: clock.wrapping_sub(idle) as i64,
    })
}

pub fn disk(path: &str) -> Option<Activity> {
    let letter: String = path
        .chars()
        .take(1)
        .flat_map(|c| c.to_uppercase())
        .collect();
    let volume = wide(&format!("\\\\.\\{}:", letter));
    let share = FILE_SHARE_READ | FILE_SHARE_WRITE;
    let access = DEVICE_ATTRIBUTES_ONLY;
    let device = unsafe {
        CreateFileW(
            volume.as_ptr(),
            access,
            share,
            null_mut(),
            OPEN_EXISTING,
            0,
            0,
        )
    };
    if device == INVALID {
        return None;
    }
    let mut perf = DiskPerformance::default();
    let mut got = 0u32;
    let ok = unsafe {
        DeviceIoControl(
            device,
            IOCTL_DISK_PERFORMANCE,
            null_mut(),
            0,
            &mut perf as *mut DiskPerformance as *mut c_void,
            size_of::<DiskPerformance>() as u32,
            &mut got,
            null_mut(),
        )
    };
    unsafe { CloseHandle(device) };
    let needed = core::mem::offset_of!(DiskPerformance, device_number);
    if ok == 0 || (got as usize) < needed {
        return None;
    }
    Some(perf.activity(letter.chars().next().map_or(0, u64::from)))
}

impl DiskPerformance {
    fn activity(&self, source: u64) -> Activity {
        Activity {
            source,
            clock: self.query_time,
            busy: self.query_time.wrapping_sub(self.idle_time),
        }
    }
}

pub fn memory() -> Option<(f64, f64)> {
    let mut status = MemoryStatus {
        length: size_of::<MemoryStatus>() as u32,
        ..MemoryStatus::default()
    };
    if unsafe { GlobalMemoryStatusEx(&mut status) } == 0 {
        return None;
    }
    Some((status.total as f64, status.available as f64))
}

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

pub fn home() -> Option<String> {
    env("USERPROFILE")
        .filter(|s| !s.is_empty())
        .or_else(|| env("HOME").filter(|s| !s.is_empty()))
}

pub fn state_dir() -> Option<String> {
    let base = env("LOCALAPPDATA")
        .filter(|s| !s.is_empty())
        .or_else(|| home().map(|h| super::join(&h, &["AppData", "Local"])))?;
    Some(super::join(&base, &["claude-statusline"]))
}

#[cfg(not(test))]
mod rt {
    use super::*;
    use core::alloc::{GlobalAlloc, Layout};
    use core::sync::atomic::{AtomicBool, Ordering};

    #[link(name = "kernel32", kind = "raw-dylib")]
    extern "system" {
        fn ExitProcess(code: u32) -> !;
        fn GetProcessHeap() -> Handle;
        fn HeapAlloc(heap: Handle, flags: u32, size: usize) -> *mut u8;
        fn HeapReAlloc(heap: Handle, flags: u32, ptr: *mut u8, size: usize) -> *mut u8;
        fn HeapFree(heap: Handle, flags: u32, ptr: *mut u8) -> i32;
    }

    const HEAP_ZERO_MEMORY: u32 = 8;
    const MEMORY_ALLOCATION_ALIGNMENT: usize = 16;
    const PANIC_EXIT_CODE: u32 = 101;
    const UNWIND_EXIT_CODE: u32 = 102;

    struct Heap;

    unsafe impl GlobalAlloc for Heap {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            if layout.align() > MEMORY_ALLOCATION_ALIGNMENT {
                return null_mut();
            }
            HeapAlloc(GetProcessHeap(), 0, layout.size())
        }
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            if layout.align() > MEMORY_ALLOCATION_ALIGNMENT {
                return null_mut();
            }
            HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, layout.size())
        }
        unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
            HeapFree(GetProcessHeap(), 0, ptr);
        }
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
            if layout.align() > MEMORY_ALLOCATION_ALIGNMENT {
                return null_mut();
            }
            HeapReAlloc(GetProcessHeap(), 0, ptr, new_size)
        }
    }

    #[global_allocator]
    static ALLOCATOR: Heap = Heap;

    struct StandardError;

    impl core::fmt::Write for StandardError {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            err(s);
            Ok(())
        }
    }

    static PANICKING: AtomicBool = AtomicBool::new(false);

    #[panic_handler]
    fn panic(info: &core::panic::PanicInfo) -> ! {
        let first = !PANICKING.swap(true, Ordering::Relaxed);
        if let (true, Some(location)) = (first, info.location()) {
            let _ = crate::messages::PANICKED.write_to(&mut StandardError, location);
        }
        unsafe { ExitProcess(PANIC_EXIT_CODE) }
    }

    #[no_mangle]
    pub static _fltused: i32 = 0;

    #[no_mangle]
    pub extern "C" fn __CxxFrameHandler3() -> ! {
        unsafe { ExitProcess(UNWIND_EXIT_CODE) }
    }

    #[cfg(target_arch = "x86_64")]
    core::arch::global_asm!(include_str!("windows_x86_64.s"));
    #[cfg(target_arch = "aarch64")]
    core::arch::global_asm!(include_str!("windows_aarch64.s"));

    #[no_mangle]
    pub extern "system" fn mainCRTStartup() -> ! {
        let code = crate::app::main(args());
        unsafe { ExitProcess(code as u32) }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        in_utf16_chunks, on_network, split_command_line, system_dir, verbatim, DiskPerformance,
    };
    use crate::state::State;
    use alloc::string::String;
    use alloc::vec::Vec;

    #[test]
    fn console_text_is_cut_between_characters() {
        const HIGH_SURROGATES: core::ops::RangeInclusive<u16> = 0xD800..=0xDBFF;
        let text = "a\u{10348}bc\u{e9}\u{10348}\u{10348}d";
        let mut buffer = [0u16; 3];
        let mut chunks: Vec<Vec<u16>> = Vec::new();
        assert!(in_utf16_chunks(text, &mut buffer, &mut |chunk| {
            chunks.push(chunk.to_vec());
            true
        }));
        assert_eq!(chunks.concat(), text.encode_utf16().collect::<Vec<u16>>());
        for chunk in &chunks {
            assert!(!chunk.is_empty() && chunk.len() <= buffer.len());
            assert!(!HIGH_SURROGATES.contains(chunk.last().unwrap()));
        }
    }

    #[test]
    fn a_failed_console_write_stops_the_text() {
        let mut calls = 0;
        let written = in_utf16_chunks("abcdef", &mut [0u16; 2], &mut |_| {
            calls += 1;
            false
        });
        assert!(!written);
        assert_eq!(calls, 1);
    }

    #[test]
    fn long_paths_take_the_verbatim_prefix_of_their_kind() {
        let cases = [
            ("C:\\deep\\settings.json", "\\\\?\\C:\\deep\\settings.json"),
            (
                "\\\\server\\share\\a.json",
                "\\\\?\\UNC\\server\\share\\a.json",
            ),
            ("\\\\?\\C:\\deep\\a.json", "\\\\?\\C:\\deep\\a.json"),
            ("\\\\.\\pipe\\name", "\\\\.\\pipe\\name"),
        ];
        for (full, expected) in cases {
            assert_eq!(verbatim(String::from(full)), expected);
        }
    }

    fn split(line: &str) -> Vec<String> {
        let units: Vec<u16> = line.encode_utf16().collect();
        split_command_line(&units)
    }

    #[test]
    fn the_program_name_drops_its_quotes_and_keeps_its_backslashes() {
        let cases: [(&str, &[&str]); 3] = [
            (
                r#""C:\Program Files\statusline.exe" --bench 10"#,
                &[r"C:\Program Files\statusline.exe", "--bench", "10"],
            ),
            (
                r#"C:\bin\"st atus"\line.exe x"#,
                &[r"C:\bin\st atus\line.exe", "x"],
            ),
            ("", &[""]),
        ];
        for (line, expected) in cases {
            assert_eq!(split(line), expected, "{}", line);
        }
    }

    #[test]
    fn arguments_follow_the_documented_quote_and_backslash_rules() {
        let cases: [(&str, &[&str]); 6] = [
            (r#"x "abc" d e"#, &["x", "abc", "d", "e"]),
            (r#"x a\\b d"e f"g h"#, &["x", r"a\\b", "de fg", "h"]),
            (r#"x a\\\"b c d"#, &["x", r#"a\"b"#, "c", "d"]),
            (r#"x a\\\\"b c" d e"#, &["x", r"a\\b c", "d", "e"]),
            (r#"x a"b"" c d"#, &["x", r#"ab" c d"#]),
            (r#"x "ab\"c" "\\" d"#, &["x", r#"ab"c"#, r"\", "d"]),
        ];
        for (line, expected) in cases {
            assert_eq!(split(line), expected, "{}", line);
        }
    }

    #[test]
    fn empty_arguments_stay_and_extra_blanks_go() {
        let cases: [(&str, &[&str]); 4] = [
            ("x \"\" a\t\t\"\"  ", &["x", "", "a", ""]),
            ("x \"\"", &["x", ""]),
            ("x \"a b", &["x", "a b"]),
            (" a  b", &["", "a", "b"]),
        ];
        for (line, expected) in cases {
            assert_eq!(split(line), expected, "{}", line);
        }
    }

    #[test]
    fn shares_are_on_the_network_and_the_system_drive_is_not() {
        let system = system_dir().unwrap();
        let verbatim_system = format!(r"\\?\{}", system);
        let cases = [
            (r"\\192.0.2.1\share", true),
            (r"//192.0.2.1/share/project", true),
            (r"\\?\UNC\192.0.2.1\share", true),
            (r"\\?\unc\192.0.2.1\share", true),
            (system.as_str(), false),
            (verbatim_system.as_str(), false),
            (r"\project", false),
            ("project", false),
            ("", false),
        ];
        for (path, expected) in cases {
            assert_eq!(on_network(path), expected, "{}", path);
        }
    }

    #[test]
    fn overlapping_requests_count_the_busy_time_once() {
        let before = DiskPerformance {
            read_time: 40_000_000,
            idle_time: 70_000_000,
            query_time: 100_000_000,
            ..Default::default()
        };
        let after = DiskPerformance {
            read_time: 70_000_000,
            idle_time: 75_000_000,
            query_time: 110_000_000,
            ..Default::default()
        };
        let mut state = State::parse("");
        assert_eq!(state.counter("io", Some(before.activity(1)), 1000.0), None);
        assert_eq!(
            state.counter("io", Some(after.activity(1)), 1005.0),
            Some(50.0)
        );
    }
}
