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
    fn GetFileAttributesExW(name: *const u16, level: u32, data: *mut FileAttributeData) -> i32;
    fn GetFileAttributesW(name: *const u16) -> u32;
    fn GetFinalPathNameByHandleW(file: Handle, buf: *mut u16, size: u32, flags: u32) -> u32;
    fn MoveFileExW(from: *const u16, to: *const u16, flags: u32) -> i32;
    fn DeleteFileW(name: *const u16) -> i32;
    fn CreateDirectoryW(name: *const u16, security: *mut c_void) -> i32;
    fn FindFirstFileW(pattern: *const u16, data: *mut FindData) -> Handle;
    fn FindNextFileW(search: Handle, data: *mut FindData) -> i32;
    fn FindClose(search: Handle) -> i32;
    fn GetCommandLineW() -> *const u16;
    fn LocalFree(memory: *mut c_void) -> *mut c_void;
    fn GetEnvironmentVariableW(name: *const u16, buf: *mut u16, size: u32) -> u32;
    fn GetModuleFileNameW(module: Handle, buf: *mut u16, size: u32) -> u32;
    fn GetSystemDirectoryW(buf: *mut u16, size: u32) -> u32;
    fn GetCurrentDirectoryW(size: u32, buf: *mut u16) -> u32;
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
}

#[link(name = "shell32", kind = "raw-dylib")]
extern "system" {
    fn CommandLineToArgvW(line: *const u16, count: *mut i32) -> *mut *mut u16;
}

#[link(name = "advapi32", kind = "raw-dylib")]
extern "system" {
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
const FILE_TYPE_DISK: u32 = 1;
const FILE_TYPE_CHAR: u32 = 2;
const MOVEFILE_REPLACE_EXISTING: u32 = 1;
const GET_FILE_EX_INFO_STANDARD: u32 = 0;
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
const DRIVE_ROOT_INDEX: usize = 2;
const MAX_BUFFER_UNITS: u32 = 1 << 20;
const READ_CHUNK_BYTES: usize = 64 << 10;
const MAX_WRITE_BYTES: usize = 1 << 30;
const CONSOLE_CHUNK_UNITS: usize = 8192;
const STDIN_SIZE_HINT: usize = 4096;
const UNIX_EPOCH_TICKS: u64 = 116_444_736_000_000_000;
const TICKS_PER_SECOND: i64 = 10_000_000;

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

unsafe fn from_ptr(text: *const u16) -> String {
    let mut len = 0;
    while *text.add(len) != 0 {
        len += 1;
    }
    String::from_utf16_lossy(core::slice::from_raw_parts(text, len))
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
    let mut out = Vec::new();
    unsafe {
        let mut count = 0i32;
        let argv = CommandLineToArgvW(GetCommandLineW(), &mut count);
        if argv.is_null() {
            return out;
        }
        for i in 0..count as usize {
            out.push(from_ptr(*argv.add(i)));
        }
        LocalFree(argv as *mut c_void);
    }
    out
}

pub fn env(name: &str) -> Option<String> {
    let wide_name = wide(name);
    with_growing_buffer(&|b, n| unsafe { GetEnvironmentVariableW(wide_name.as_ptr(), b, n) })
}

fn open_read(path: &str) -> Handle {
    let wide_path = wide(path);
    unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            GENERIC_READ,
            SHARE_ALL,
            null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            0,
        )
    }
}

fn read_all(file: Handle, hint: usize, limit: usize) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(hint + 1);
    let mut chunk = vec![0u8; READ_CHUNK_BYTES];
    while out.len() <= limit {
        let mut got = 0u32;
        let ok = unsafe {
            ReadFile(
                file,
                chunk.as_mut_ptr(),
                chunk.len() as u32,
                &mut got,
                null_mut(),
            )
        };
        if ok == 0 || got == 0 {
            break;
        }
        out.extend_from_slice(&chunk[..got as usize]);
    }
    out
}

pub fn read_limited(path: &str, limit: usize) -> Option<Vec<u8>> {
    let file = open_read(path);
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
    let wide_path = wide(path);
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
    let wide_path = wide(path);
    let attributes = unsafe { GetFileAttributesW(wide_path.as_ptr()) };
    let linked = attributes != u32::MAX && attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0;
    let file = if linked { open_read(path) } else { INVALID };
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
    let tmp = format!("{}.{}.tmp", target, unsafe { GetCurrentProcessId() });
    if write(&tmp, data) {
        if rename(&tmp, &target) {
            return true;
        }
        remove(&tmp);
    }
    write(&target, data)
}

pub fn remove(path: &str) -> bool {
    let wide_path = wide(path);
    unsafe { DeleteFileW(wide_path.as_ptr()) != 0 }
}

pub fn rename(from: &str, to: &str) -> bool {
    let (wide_from, wide_to) = (wide(from), wide(to));
    let flags = MOVEFILE_REPLACE_EXISTING;
    unsafe { MoveFileExW(wide_from.as_ptr(), wide_to.as_ptr(), flags) != 0 }
}

pub fn create_private(path: &str, data: &[u8]) -> bool {
    write_with(path, SHARE_NONE, CREATE_NEW, data)
}

pub fn make_executable(path: &str) -> bool {
    exists(path)
}

pub fn stamp(path: &str) -> Option<(u64, i64)> {
    let wide_path = wide(path);
    let mut data = FileAttributeData::default();
    let level = GET_FILE_EX_INFO_STANDARD;
    if unsafe { GetFileAttributesExW(wide_path.as_ptr(), level, &mut data) } == 0 {
        return None;
    }
    let since_epoch = data.written.ticks() as i64 - UNIX_EPOCH_TICKS as i64;
    let size = (data.size_high as u64) << 32 | data.size_low as u64;
    Some((size, since_epoch.div_euclid(TICKS_PER_SECOND)))
}

pub fn exists(path: &str) -> bool {
    let wide_path = wide(path);
    unsafe { GetFileAttributesW(wide_path.as_ptr()) != u32::MAX }
}

pub fn mkdirs(path: &str) -> bool {
    for (i, c) in path.char_indices() {
        if (c == '\\' || c == '/') && i > DRIVE_ROOT_INDEX {
            let wide_parent = wide(&path[..i]);
            unsafe { CreateDirectoryW(wide_parent.as_ptr(), null_mut()) };
        }
    }
    let wide_path = wide(path);
    unsafe { CreateDirectoryW(wide_path.as_ptr(), null_mut()) };
    exists(path)
}

pub fn list(dir: &str, prefix: &str) -> Vec<String> {
    let mut out = Vec::new();
    let pattern = wide(&format!("{}\\{}*", dir, prefix));
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

fn write_console(console: Handle, s: &str) {
    let units: Vec<u16> = s.encode_utf16().collect();
    let mut done = 0usize;
    while done < units.len() {
        let mut written = 0u32;
        let len = core::cmp::min(units.len() - done, CONSOLE_CHUNK_UNITS) as u32;
        let text = unsafe { units.as_ptr().add(done) };
        if unsafe { WriteConsoleW(console, text, len, &mut written, null_mut()) } == 0
            || written == 0
        {
            return;
        }
        done += written as usize;
    }
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

pub fn cpu() -> Option<(i64, i64)> {
    let (mut idle, mut kernel, mut user) = (0u64, 0u64, 0u64);
    if unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) } == 0 {
        return None;
    }
    let clock = kernel.wrapping_add(user);
    Some((clock as i64, clock.wrapping_sub(idle) as i64))
}

pub fn disk(path: &str) -> Option<(i64, i64)> {
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
    Some((
        perf.query_time,
        perf.read_time.wrapping_add(perf.write_time),
    ))
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

    #[panic_handler]
    fn panic(_: &core::panic::PanicInfo) -> ! {
        unsafe { ExitProcess(PANIC_EXIT_CODE) }
    }

    #[no_mangle]
    pub static _fltused: i32 = 0;

    #[no_mangle]
    pub extern "C" fn __CxxFrameHandler3() -> ! {
        unsafe { ExitProcess(UNWIND_EXIT_CODE) }
    }

    core::arch::global_asm!(
        ".globl memcpy",
        "memcpy:",
        "mov rax, rcx",
        "push rdi",
        "push rsi",
        "mov rdi, rcx",
        "mov rsi, rdx",
        "mov rcx, r8",
        "rep movsb",
        "pop rsi",
        "pop rdi",
        "ret",
        ".globl memmove",
        "memmove:",
        "mov rax, rcx",
        "cmp rcx, rdx",
        "jbe memmove_up",
        "lea r9, [rdx + r8]",
        "cmp rcx, r9",
        "jae memmove_up",
        "push rdi",
        "push rsi",
        "lea rdi, [rcx + r8 - 1]",
        "lea rsi, [rdx + r8 - 1]",
        "mov rcx, r8",
        "std",
        "rep movsb",
        "cld",
        "pop rsi",
        "pop rdi",
        "ret",
        "memmove_up:",
        "push rdi",
        "push rsi",
        "mov rdi, rcx",
        "mov rsi, rdx",
        "mov rcx, r8",
        "rep movsb",
        "pop rsi",
        "pop rdi",
        "ret",
        ".globl memset",
        "memset:",
        "mov r9, rcx",
        "push rdi",
        "mov rdi, rcx",
        "mov eax, edx",
        "mov rcx, r8",
        "rep stosb",
        "pop rdi",
        "mov rax, r9",
        "ret",
        ".globl memcmp",
        ".globl bcmp",
        "bcmp:",
        "memcmp:",
        "xor eax, eax",
        "test r8, r8",
        "jz memcmp_end",
        "memcmp_loop:",
        "movzx eax, byte ptr [rcx]",
        "movzx r9d, byte ptr [rdx]",
        "sub eax, r9d",
        "jnz memcmp_end",
        "inc rcx",
        "inc rdx",
        "dec r8",
        "jnz memcmp_loop",
        "memcmp_end:",
        "ret",
        ".globl wcslen",
        "wcslen:",
        "xor eax, eax",
        "wcslen_loop:",
        "cmp word ptr [rcx + rax*2], 0",
        "je wcslen_end",
        "inc rax",
        "jmp wcslen_loop",
        "wcslen_end:",
        "ret",
        ".globl __chkstk",
        "__chkstk:",
        "push rcx",
        "push rax",
        "lea rcx, [rsp + 24]",
        "cmp rax, 0x1000",
        "jb chkstk_last",
        "chkstk_loop:",
        "sub rcx, 0x1000",
        "test byte ptr [rcx], 0",
        "sub rax, 0x1000",
        "cmp rax, 0x1000",
        "ja chkstk_loop",
        "chkstk_last:",
        "sub rcx, rax",
        "test byte ptr [rcx], 0",
        "pop rax",
        "pop rcx",
        "ret",
    );

    #[no_mangle]
    pub extern "system" fn start() -> ! {
        let code = crate::app::main(args());
        unsafe { ExitProcess(code as u32) }
    }
}
