// Couche systeme Windows sans bibliotheque standard : appels Win32 directs,
// lies sans CRT pour un binaire de quelques dizaines de Ko qui demarre vite.

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
    fn GetStdHandle(n: u32) -> Handle;
    fn GetFileType(h: Handle) -> u32;
    fn GetConsoleMode(h: Handle, m: *mut u32) -> i32;
    fn SetConsoleMode(h: Handle, m: u32) -> i32;
    fn WriteConsoleW(h: Handle, b: *const u16, n: u32, w: *mut u32, r: *mut c_void) -> i32;
    fn ReadFile(h: Handle, b: *mut u8, n: u32, r: *mut u32, o: *mut c_void) -> i32;
    fn WriteFile(h: Handle, b: *const u8, n: u32, w: *mut u32, o: *mut c_void) -> i32;
    fn CreateFileW(name: *const u16, access: u32, share: u32, sa: *mut c_void, disp: u32, flags: u32, tmpl: Handle) -> Handle;
    fn CloseHandle(h: Handle) -> i32;
    fn GetFileSizeEx(h: Handle, size: *mut i64) -> i32;
    fn GetFileAttributesExW(name: *const u16, level: u32, data: *mut u32) -> i32;
    fn GetFileAttributesW(name: *const u16) -> u32;
    fn MoveFileExW(from: *const u16, to: *const u16, flags: u32) -> i32;
    fn DeleteFileW(name: *const u16) -> i32;
    fn CreateDirectoryW(name: *const u16, sa: *mut c_void) -> i32;
    fn FindFirstFileW(pattern: *const u16, data: *mut u32) -> Handle;
    fn FindNextFileW(h: Handle, data: *mut u32) -> i32;
    fn FindClose(h: Handle) -> i32;
    fn GetCommandLineW() -> *const u16;
    fn LocalFree(p: *mut c_void) -> *mut c_void;
    fn GetEnvironmentVariableW(name: *const u16, buf: *mut u16, size: u32) -> u32;
    fn GetModuleFileNameW(m: Handle, buf: *mut u16, size: u32) -> u32;
    fn GetCurrentDirectoryW(size: u32, buf: *mut u16) -> u32;
    fn GetCurrentProcessId() -> u32;
    fn GetSystemTimePreciseAsFileTime(ft: *mut u64);
    fn QueryPerformanceCounter(c: *mut i64) -> i32;
    fn QueryPerformanceFrequency(f: *mut i64) -> i32;
    fn GetSystemTimes(idle: *mut u64, kernel: *mut u64, user: *mut u64) -> i32;
    fn GlobalMemoryStatusEx(m: *mut u64) -> i32;
    fn DeviceIoControl(h: Handle, code: u32, i: *mut c_void, il: u32, o: *mut c_void, ol: u32, r: *mut u32, ov: *mut c_void) -> i32;
    fn ExitProcess(code: u32) -> !;
    fn GetProcessHeap() -> Handle;
    fn HeapAlloc(h: Handle, flags: u32, n: usize) -> *mut u8;
    fn HeapReAlloc(h: Handle, flags: u32, p: *mut u8, n: usize) -> *mut u8;
    fn HeapFree(h: Handle, flags: u32, p: *mut u8) -> i32;
}

#[link(name = "shell32", kind = "raw-dylib")]
extern "system" {
    fn CommandLineToArgvW(line: *const u16, n: *mut i32) -> *mut *mut u16;
}

#[link(name = "advapi32", kind = "raw-dylib")]
extern "system" {
    fn RegGetValueW(key: isize, sub: *const u16, value: *const u16, flags: u32, kind: *mut u32, data: *mut c_void, size: *mut u32) -> i32;
}

const GENERIC_READ: u32 = 0x8000_0000;
const GENERIC_WRITE: u32 = 0x4000_0000;
const SHARE_ALL: u32 = 7;
const OPEN_EXISTING: u32 = 3;
const CREATE_ALWAYS: u32 = 2;
const EPOCH: u64 = 116_444_736_000_000_000;

fn wide(s: &str) -> Vec<u16> {
    let mut v: Vec<u16> = s.encode_utf16().collect();
    v.push(0);
    v
}

unsafe fn from_ptr(p: *const u16) -> String {
    let mut n = 0;
    while *p.add(n) != 0 {
        n += 1;
    }
    String::from_utf16_lossy(core::slice::from_raw_parts(p, n))
}

// Remplit un tampon de taille croissante tant que l'appel demande plus de place
fn grow(f: &dyn Fn(*mut u16, u32) -> u32) -> Option<String> {
    let mut size = 260u32;
    loop {
        let mut buf: Vec<u16> = vec![0; size as usize];
        let n = f(buf.as_mut_ptr(), size);
        if n == 0 {
            return None;
        }
        if n < size {
            return Some(String::from_utf16_lossy(&buf[..n as usize]));
        }
        size = if n > size { n + 1 } else { size * 2 };
        if size > 1 << 20 {
            return None;
        }
    }
}

pub fn args() -> Vec<String> {
    let mut out = Vec::new();
    unsafe {
        let mut n = 0i32;
        let v = CommandLineToArgvW(GetCommandLineW(), &mut n);
        if v.is_null() {
            return out;
        }
        for i in 0..n as usize {
            out.push(from_ptr(*v.add(i)));
        }
        LocalFree(v as *mut c_void);
    }
    out
}

pub fn env(name: &str) -> Option<String> {
    let w = wide(name);
    grow(&|b, n| unsafe { GetEnvironmentVariableW(w.as_ptr(), b, n) })
}

fn open_read(path: &str) -> Handle {
    let w = wide(path);
    unsafe { CreateFileW(w.as_ptr(), GENERIC_READ, SHARE_ALL, null_mut(), OPEN_EXISTING, 0x80, 0) }
}

fn read_all(h: Handle, hint: usize) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(hint + 1);
    let mut chunk = vec![0u8; 65536];
    loop {
        let mut got = 0u32;
        let ok = unsafe { ReadFile(h, chunk.as_mut_ptr(), chunk.len() as u32, &mut got, null_mut()) };
        if ok == 0 || got == 0 {
            break;
        }
        out.extend_from_slice(&chunk[..got as usize]);
    }
    out
}

pub fn read(path: &str) -> Option<Vec<u8>> {
    let h = open_read(path);
    if h == INVALID {
        return None;
    }
    let mut size = 0i64;
    unsafe { GetFileSizeEx(h, &mut size) };
    let data = read_all(h, if size > 0 { size as usize } else { 0 });
    unsafe { CloseHandle(h) };
    Some(data)
}

fn write_handle(h: Handle, data: &[u8]) -> bool {
    let mut done = 0usize;
    while done < data.len() {
        let mut w = 0u32;
        let n = core::cmp::min(data.len() - done, 1 << 30) as u32;
        let ok = unsafe { WriteFile(h, data.as_ptr().add(done), n, &mut w, null_mut()) };
        if ok == 0 || w == 0 {
            return false;
        }
        done += w as usize;
    }
    true
}

pub fn write(path: &str, data: &[u8]) -> bool {
    let w = wide(path);
    let h = unsafe { CreateFileW(w.as_ptr(), GENERIC_WRITE, 1, null_mut(), CREATE_ALWAYS, 0x80, 0) };
    if h == INVALID {
        return false;
    }
    let ok = write_handle(h, data);
    unsafe { CloseHandle(h) };
    ok
}

// Ecrit a cote puis renomme, pour qu'un lecteur concurrent ne voie jamais un fichier tronque
pub fn write_atomic(path: &str, data: &[u8]) -> bool {
    let tmp = format!("{}.{}.tmp", path, unsafe { GetCurrentProcessId() });
    let wt = wide(&tmp);
    if write(&tmp, data) {
        let wp = wide(path);
        if unsafe { MoveFileExW(wt.as_ptr(), wp.as_ptr(), 1) } != 0 {
            return true;
        }
        unsafe { DeleteFileW(wt.as_ptr()) };
    }
    write(path, data)
}

pub fn remove(path: &str) -> bool {
    let w = wide(path);
    unsafe { DeleteFileW(w.as_ptr()) != 0 }
}

// (taille, date de modification en secondes Unix)
pub fn stamp(path: &str) -> Option<(u64, i64)> {
    let w = wide(path);
    let mut d = [0u32; 9];
    if unsafe { GetFileAttributesExW(w.as_ptr(), 0, d.as_mut_ptr()) } == 0 {
        return None;
    }
    let ft = (d[6] as u64) << 32 | d[5] as u64;
    let secs = (ft as i64 - EPOCH as i64).div_euclid(10_000_000);
    Some(((d[7] as u64) << 32 | d[8] as u64, secs))
}

pub fn exists(path: &str) -> bool {
    let w = wide(path);
    unsafe { GetFileAttributesW(w.as_ptr()) != u32::MAX }
}

pub fn mkdirs(path: &str) -> bool {
    for (i, c) in path.char_indices() {
        if (c == '\\' || c == '/') && i > 2 {
            let w = wide(&path[..i]);
            unsafe { CreateDirectoryW(w.as_ptr(), null_mut()) };
        }
    }
    let w = wide(path);
    unsafe { CreateDirectoryW(w.as_ptr(), null_mut()) };
    exists(path)
}

// Noms des entrees de dir qui commencent par prefix
pub fn list(dir: &str, prefix: &str) -> Vec<String> {
    let mut out = Vec::new();
    let w = wide(&format!("{}\\{}*", dir, prefix));
    let mut d = [0u32; 148];
    let h = unsafe { FindFirstFileW(w.as_ptr(), d.as_mut_ptr()) };
    if h == INVALID {
        return out;
    }
    loop {
        let name = unsafe { from_ptr((d.as_ptr() as *const u16).add(22)) };
        if name != "." && name != ".." {
            out.push(name);
        }
        if unsafe { FindNextFileW(h, d.as_mut_ptr()) } == 0 {
            break;
        }
    }
    unsafe { FindClose(h) };
    out
}

// Rien a lire quand l'entree est une console : on ne bloque pas
pub fn stdin() -> Option<String> {
    let h = unsafe { GetStdHandle(-10i32 as u32) };
    if h == 0 || h == INVALID || unsafe { GetFileType(h) } == 2 {
        return None;
    }
    let data = read_all(h, 4096);
    Some(String::from_utf8_lossy(&data).into_owned())
}

fn put(n: i32, s: &str) {
    let h = unsafe { GetStdHandle(n as u32) };
    if h == 0 || h == INVALID {
        return;
    }
    let mut mode = 0u32;
    if unsafe { GetConsoleMode(h, &mut mode) } != 0 {
        unsafe { SetConsoleMode(h, mode | 4) };
        let w: Vec<u16> = s.encode_utf16().collect();
        let mut done = 0usize;
        while done < w.len() {
            let mut n = 0u32;
            let len = core::cmp::min(w.len() - done, 8192) as u32;
            if unsafe { WriteConsoleW(h, w.as_ptr().add(done), len, &mut n, null_mut()) } == 0 || n == 0 {
                return;
            }
            done += n as usize;
        }
    } else {
        write_handle(h, s.as_bytes());
    }
}

pub fn out(s: &str) {
    put(-11, s);
}

pub fn err(s: &str) {
    put(-12, s);
}

pub fn exe() -> Option<String> {
    grow(&|b, n| unsafe { GetModuleFileNameW(0, b, n) })
}

pub fn cwd() -> Option<String> {
    grow(&|b, n| unsafe { GetCurrentDirectoryW(n, b) })
}

pub fn now() -> f64 {
    let mut ft = 0u64;
    unsafe { GetSystemTimePreciseAsFileTime(&mut ft) };
    (ft as f64 - EPOCH as f64) / 1e7
}

pub fn ticks() -> f64 {
    let (mut c, mut f) = (0i64, 0i64);
    unsafe {
        QueryPerformanceCounter(&mut c);
        QueryPerformanceFrequency(&mut f);
    }
    if f == 0 {
        0.0
    } else {
        c as f64 / f as f64
    }
}

// (horloge, temps occupe) en centaines de nanosecondes
pub fn cpu() -> Option<(i64, i64)> {
    let (mut idle, mut kernel, mut user) = (0u64, 0u64, 0u64);
    if unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) } == 0 {
        return None;
    }
    let clock = kernel.wrapping_add(user);
    Some((clock as i64, clock.wrapping_sub(idle) as i64))
}

// IOCTL_DISK_PERFORMANCE sur le volume : (QueryTime, ReadTime + WriteTime)
pub fn disk(path: &str) -> Option<(i64, i64)> {
    let letter: String = path.chars().take(1).flat_map(|c| c.to_uppercase()).collect();
    let w = wide(&format!("\\\\.\\{}:", letter));
    let h = unsafe { CreateFileW(w.as_ptr(), 0, 3, null_mut(), OPEN_EXISTING, 0, 0) };
    if h == INVALID {
        return None;
    }
    let mut p = [0i64; 11];
    let mut got = 0u32;
    let ok = unsafe { DeviceIoControl(h, 0x70020, null_mut(), 0, p.as_mut_ptr() as *mut c_void, 88, &mut got, null_mut()) };
    unsafe { CloseHandle(h) };
    if ok == 0 || got < 64 {
        return None;
    }
    Some((p[7], p[2].wrapping_add(p[3])))
}

// (total, disponible) en octets
pub fn memory() -> Option<(f64, f64)> {
    let mut m = [0u64; 8];
    m[0] = 64;
    if unsafe { GlobalMemoryStatusEx(m.as_mut_ptr()) } == 0 {
        return None;
    }
    Some((m[1] as f64, m[2] as f64))
}

pub fn os_theme() -> Option<bool> {
    let key = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
    let name = wide("AppsUseLightTheme");
    let mut v = 0u32;
    let mut size = 4u32;
    let hkcu = 0x8000_0001u32 as i32 as isize;
    let r = unsafe { RegGetValueW(hkcu, key.as_ptr(), name.as_ptr(), 0x10, null_mut(), &mut v as *mut u32 as *mut c_void, &mut size) };
    match r {
        0 => Some(v != 0),
        // Valeur jamais posee : Windows est en mode clair par defaut
        2 => Some(true),
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
    env("USERPROFILE").filter(|s| !s.is_empty()).or_else(|| env("HOME").filter(|s| !s.is_empty()))
}

pub fn state_dir() -> Option<String> {
    let base = env("LOCALAPPDATA")
        .filter(|s| !s.is_empty())
        .or_else(|| home().map(|h| super::join(&h, &["AppData", "Local"])))?;
    Some(super::join(&base, &["claude-statusline"]))
}

// Tout ce que la CRT fournirait : allocateur, panique, fonctions memoire, point d'entree
#[cfg(not(test))]
mod rt {
    use super::*;
    use core::alloc::{GlobalAlloc, Layout};

    struct Heap;

    unsafe impl GlobalAlloc for Heap {
        unsafe fn alloc(&self, l: Layout) -> *mut u8 {
            if l.align() > 16 {
                return null_mut();
            }
            HeapAlloc(GetProcessHeap(), 0, l.size())
        }
        unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
            if l.align() > 16 {
                return null_mut();
            }
            HeapAlloc(GetProcessHeap(), 8, l.size())
        }
        unsafe fn dealloc(&self, p: *mut u8, _l: Layout) {
            HeapFree(GetProcessHeap(), 0, p);
        }
        unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
            if l.align() > 16 {
                return null_mut();
            }
            HeapReAlloc(GetProcessHeap(), 0, p, n)
        }
    }

    #[global_allocator]
    static A: Heap = Heap;

    #[panic_handler]
    fn panic(_: &core::panic::PanicInfo) -> ! {
        unsafe { ExitProcess(101) }
    }

    #[no_mangle]
    pub static _fltused: i32 = 0;

    #[no_mangle]
    pub extern "C" fn __CxxFrameHandler3() -> ! {
        unsafe { ExitProcess(102) }
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
        // Sonde de pile pour les cadres de plus d'une page
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

