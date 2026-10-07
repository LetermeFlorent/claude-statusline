// Couche systeme Linux et macOS, sur la bibliotheque standard

use std::io::{IsTerminal, Read, Write};
use std::string::String;
use std::vec::Vec;

pub const SEP: char = '/';

pub fn args() -> Vec<String> {
    std::env::args_os().map(|a| a.to_string_lossy().into_owned()).collect()
}

pub fn env(name: &str) -> Option<String> {
    std::env::var_os(name).map(|v| v.to_string_lossy().into_owned()).filter(|s| !s.is_empty())
}

pub fn read(path: &str) -> Option<Vec<u8>> {
    std::fs::read(path).ok()
}

pub fn write(path: &str, data: &[u8]) -> bool {
    std::fs::write(path, data).is_ok()
}

// Ecrit a cote puis renomme, pour qu'un lecteur concurrent ne voie jamais un fichier tronque
pub fn write_atomic(path: &str, data: &[u8]) -> bool {
    let tmp = format!("{}.{}.tmp", path, std::process::id());
    if write(&tmp, data) {
        if std::fs::rename(&tmp, path).is_ok() {
            return true;
        }
        let _ = std::fs::remove_file(&tmp);
    }
    write(path, data)
}

pub fn remove(path: &str) -> bool {
    std::fs::remove_file(path).is_ok()
}

// (taille, date de modification en secondes Unix)
pub fn stamp(path: &str) -> Option<(u64, i64)> {
    use std::os::unix::fs::MetadataExt;
    let m = std::fs::metadata(path).ok()?;
    Some((m.len(), m.mtime()))
}

pub fn exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}

pub fn mkdirs(path: &str) -> bool {
    std::fs::create_dir_all(path).is_ok()
}

pub fn list(dir: &str, prefix: &str) -> Vec<String> {
    let Ok(rd) = std::fs::read_dir(dir) else { return Vec::new() };
    rd.filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with(prefix))
        .collect()
}

// Rien a lire quand l'entree est un terminal : on ne bloque pas
pub fn stdin() -> Option<String> {
    let mut i = std::io::stdin();
    if i.is_terminal() {
        return None;
    }
    let mut data = Vec::new();
    let _ = i.read_to_end(&mut data);
    Some(String::from_utf8_lossy(&data).into_owned())
}

pub fn out(s: &str) {
    let mut o = std::io::stdout().lock();
    let _ = o.write_all(s.as_bytes());
    let _ = o.flush();
}

pub fn err(s: &str) {
    let _ = std::io::stderr().write_all(s.as_bytes());
}

pub fn exe() -> Option<String> {
    std::env::current_exe().ok().map(|p| p.to_string_lossy().into_owned())
}

pub fn cwd() -> Option<String> {
    std::env::current_dir().ok().map(|p| p.to_string_lossy().into_owned())
}

pub fn now() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

pub fn ticks() -> f64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START.get_or_init(std::time::Instant::now).elapsed().as_secs_f64()
}

pub fn home() -> Option<String> {
    env("HOME")
}

pub fn state_dir() -> Option<String> {
    if cfg!(target_os = "macos") {
        return home().map(|h| super::join(&h, &["Library", "Application Support", "claude-statusline"]));
    }
    let base = env("XDG_STATE_HOME").or_else(|| home().map(|h| super::join(&h, &[".local", "state"])))?;
    Some(super::join(&base, &["claude-statusline"]))
}

#[cfg(target_os = "linux")]
fn config_home() -> Option<String> {
    env("XDG_CONFIG_HOME").or_else(|| home().map(|h| super::join(&h, &[".config"])))
}

fn stamp_text(path: Option<String>) -> String {
    match path.and_then(|p| stamp(&p)) {
        Some((s, m)) => format!("{}:{}", s, m),
        None => String::from("0"),
    }
}

#[cfg(target_os = "linux")]
mod sys {
    use super::*;

    fn read_str(p: &str) -> Option<String> {
        std::fs::read_to_string(p).ok()
    }

    // Jiffies : (total hors invite, total moins repos et attente disque)
    pub fn cpu() -> Option<(i64, i64)> {
        let s = read_str("/proc/stat")?;
        let line = s.lines().next()?;
        let mut it = line.split_whitespace();
        if it.next()? != "cpu" {
            return None;
        }
        let v: Vec<i64> = it.take(8).filter_map(|x| x.parse().ok()).collect();
        if v.len() < 5 {
            return None;
        }
        let clock: i64 = v.iter().sum();
        Some((clock, clock - v[3] - v[4]))
    }

    fn dev_numbers(path: &str) -> Option<(u64, u64)> {
        use std::os::unix::fs::MetadataExt;
        let d = std::fs::metadata(path).ok()?.dev();
        let major = ((d >> 8) & 0xfff) | ((d >> 32) & 0xffff_f000);
        let minor = (d & 0xff) | ((d >> 12) & 0xffff_ff00);
        Some((major, minor))
    }

    // Millisecondes : (temps ecoule, temps passe a faire des entrees-sorties)
    pub fn disk(path: &str) -> Option<(i64, i64)> {
        let up = read_str("/proc/uptime")?;
        let secs: f64 = up.split_whitespace().next()?.parse().ok()?;
        let stats = read_str("/proc/diskstats")?;
        let want = dev_numbers(path);
        let mut fallback: Option<i64> = None;
        for line in stats.lines() {
            let f: Vec<&str> = line.split_whitespace().collect();
            if f.len() < 13 {
                continue;
            }
            let ticks: i64 = match f[12].parse() {
                Ok(t) => t,
                Err(_) => continue,
            };
            let num = (f[0].parse::<u64>().ok(), f[1].parse::<u64>().ok());
            if let (Some((a, b)), (Some(x), Some(y))) = (want, num) {
                if a == x && b == y {
                    return Some(((secs * 1000.0) as i64, ticks));
                }
            }
            let name = f[2];
            let virt = ["loop", "ram", "zram", "sr", "fd", "nbd"].iter().any(|p| name.starts_with(p));
            if fallback.is_none() && !virt && std::path::Path::new(&format!("/sys/block/{}", name)).exists() {
                fallback = Some(ticks);
            }
        }
        fallback.map(|t| ((secs * 1000.0) as i64, t))
    }

    pub fn memory() -> Option<(f64, f64)> {
        let s = read_str("/proc/meminfo")?;
        let get = |k: &str| -> Option<f64> {
            let line = s.lines().find(|l| l.starts_with(k))?;
            let kb: f64 = line[k.len()..].trim().trim_end_matches("kB").trim().parse().ok()?;
            Some(kb * 1024.0)
        };
        Some((get("MemTotal:")?, get("MemAvailable:")?))
    }

    fn gsettings(key: &str) -> Option<String> {
        let o = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", key])
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        if !o.status.success() {
            return None;
        }
        Some(String::from_utf8_lossy(&o.stdout).trim().trim_matches('\'').to_lowercase())
    }

    fn kde() -> Option<bool> {
        let desk = env("XDG_CURRENT_DESKTOP").unwrap_or_default().to_uppercase();
        if !desk.contains("KDE") {
            return None;
        }
        let text = read_str(&super::super::join(&config_home()?, &["kdeglobals"]))?;
        let mut section = "";
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                section = line;
                continue;
            }
            if section == "[Colors:Window]" {
                if let Some(v) = line.strip_prefix("BackgroundNormal=") {
                    let c: Vec<f64> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                    if c.len() >= 3 {
                        return Some(crate::theme::luminance_light([c[0], c[1], c[2]]));
                    }
                }
            }
        }
        None
    }

    pub fn os_theme() -> Option<bool> {
        if let Some(t) = env("GTK_THEME") {
            return Some(!t.to_lowercase().contains("dark"));
        }
        if let Some(b) = kde() {
            return Some(b);
        }
        match gsettings("color-scheme").as_deref() {
            Some("prefer-dark") => Some(false),
            Some("prefer-light") => Some(true),
            Some(_) => gsettings("gtk-theme").map(|t| !t.contains("dark")),
            None => None,
        }
    }

    pub fn os_stamp() -> String {
        let ch = config_home();
        format!(
            "{}|{}|{}|{}",
            env("GTK_THEME").unwrap_or_default(),
            env("XDG_CURRENT_DESKTOP").unwrap_or_default(),
            stamp_text(ch.as_ref().map(|c| super::super::join(c, &["dconf", "user"]))),
            stamp_text(ch.as_ref().map(|c| super::super::join(c, &["kdeglobals"]))),
        )
    }
}

#[cfg(target_os = "macos")]
mod sys {
    use super::*;
    use std::ffi::c_void;

    type Port = u32;

    extern "C" {
        fn mach_host_self() -> Port;
        fn host_statistics(host: Port, flavor: i32, info: *mut i32, count: *mut u32) -> i32;
        fn host_statistics64(host: Port, flavor: i32, info: *mut i32, count: *mut u32) -> i32;
        fn host_page_size(host: Port, size: *mut usize) -> i32;
        fn sysctlbyname(name: *const u8, old: *mut c_void, oldlen: *mut usize, new: *mut c_void, newlen: usize) -> i32;
        fn clock_gettime_nsec_np(clock: u32) -> u64;
    }

    #[link(name = "IOKit", kind = "framework")]
    extern "C" {
        fn IOServiceMatching(name: *const u8) -> *mut c_void;
        fn IOServiceGetMatchingServices(main: Port, matching: *mut c_void, iter: *mut u32) -> i32;
        fn IOIteratorNext(iter: u32) -> u32;
        fn IORegistryEntryCreateCFProperty(entry: u32, key: *const c_void, alloc: *const c_void, options: u32) -> *const c_void;
        fn IOObjectRelease(obj: u32) -> i32;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFStringCreateWithCString(alloc: *const c_void, s: *const u8, enc: u32) -> *const c_void;
        fn CFDictionaryGetValue(d: *const c_void, key: *const c_void) -> *const c_void;
        fn CFNumberGetValue(n: *const c_void, kind: i32, out: *mut c_void) -> u8;
        fn CFGetTypeID(cf: *const c_void) -> usize;
        fn CFDictionaryGetTypeID() -> usize;
        fn CFNumberGetTypeID() -> usize;
        fn CFRelease(cf: *const c_void);
    }

    // Ticks : (total, total moins repos)
    pub fn cpu() -> Option<(i64, i64)> {
        let mut t = [0i32; 4];
        let mut n = 4u32;
        if unsafe { host_statistics(mach_host_self(), 3, t.as_mut_ptr(), &mut n) } != 0 {
            return None;
        }
        let v: Vec<i64> = t.iter().map(|x| *x as u32 as i64).collect();
        let clock: i64 = v.iter().sum();
        Some((clock, clock - v[2]))
    }

    fn cfstr(s: &[u8]) -> *const c_void {
        unsafe { CFStringCreateWithCString(std::ptr::null(), s.as_ptr(), 0x0800_0100) }
    }

    fn number(d: *const c_void, key: *const c_void) -> i64 {
        let v = unsafe { CFDictionaryGetValue(d, key) };
        let mut out = 0i64;
        if !v.is_null() && unsafe { CFGetTypeID(v) == CFNumberGetTypeID() } {
            unsafe { CFNumberGetValue(v, 4, &mut out as *mut i64 as *mut c_void) };
        }
        out
    }

    // Nanosecondes : (temps ecoule, temps de lecture et d'ecriture de tous les disques)
    pub fn disk(_path: &str) -> Option<(i64, i64)> {
        let mut iter = 0u32;
        let matching = unsafe { IOServiceMatching(b"IOBlockStorageDriver\0".as_ptr()) };
        if matching.is_null() || unsafe { IOServiceGetMatchingServices(0, matching, &mut iter) } != 0 {
            return None;
        }
        let k_stats = cfstr(b"Statistics\0");
        let k_read = cfstr(b"Total Time (Read)\0");
        let k_write = cfstr(b"Total Time (Write)\0");
        let mut busy = 0i64;
        let mut seen = false;
        loop {
            let e = unsafe { IOIteratorNext(iter) };
            if e == 0 {
                break;
            }
            let d = unsafe { IORegistryEntryCreateCFProperty(e, k_stats, std::ptr::null(), 0) };
            if !d.is_null() {
                if unsafe { CFGetTypeID(d) == CFDictionaryGetTypeID() } {
                    busy = busy.wrapping_add(number(d, k_read)).wrapping_add(number(d, k_write));
                    seen = true;
                }
                unsafe { CFRelease(d) };
            }
            unsafe { IOObjectRelease(e) };
        }
        unsafe {
            IOObjectRelease(iter);
            CFRelease(k_stats);
            CFRelease(k_read);
            CFRelease(k_write);
        }
        if !seen {
            return None;
        }
        Some((unsafe { clock_gettime_nsec_np(8) } as i64, busy))
    }

    // Memoire utilisee comme le Moniteur d'activite : applications, fixee et compressee
    pub fn memory() -> Option<(f64, f64)> {
        let mut total = 0u64;
        let mut len = 8usize;
        let r = unsafe { sysctlbyname(b"hw.memsize\0".as_ptr(), &mut total as *mut u64 as *mut c_void, &mut len, std::ptr::null_mut(), 0) };
        if r != 0 || total == 0 {
            return None;
        }
        let host = unsafe { mach_host_self() };
        let mut page = 0usize;
        if unsafe { host_page_size(host, &mut page) } != 0 {
            return None;
        }
        let mut v = [0i32; 38];
        let mut n = 38u32;
        if unsafe { host_statistics64(host, 4, v.as_mut_ptr(), &mut n) } != 0 {
            return None;
        }
        let at = |off: usize| v[off / 4] as u32 as f64;
        let pages = at(140) - at(88) + at(12) + at(128);
        let used = pages * page as f64;
        let used = if used > total as f64 { total as f64 } else { used };
        Some((total as f64, total as f64 - used))
    }

    fn prefs() -> Option<String> {
        home().map(|h| super::super::join(&h, &["Library", "Preferences", ".GlobalPreferences.plist"]))
    }

    fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
        hay.windows(needle.len()).position(|w| w == needle)
    }

    pub fn os_theme() -> Option<bool> {
        let data = read(&prefs()?)?;
        if data.starts_with(b"bplist") {
            let mut key = vec![0x5F, 0x10, 0x13];
            key.extend_from_slice(b"AppleInterfaceStyle");
            let mut dark = vec![0x54];
            dark.extend_from_slice(b"Dark");
            return Some(!(find(&data, &key).is_some() && find(&data, &dark).is_some()));
        }
        let text = String::from_utf8_lossy(&data);
        let Some(i) = text.find("<key>AppleInterfaceStyle</key>") else { return Some(true) };
        Some(!text[i..].trim_start_matches("<key>AppleInterfaceStyle</key>").trim_start().starts_with("<string>Dark</string>"))
    }

    pub fn os_stamp() -> String {
        stamp_text(prefs())
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod sys {
    pub fn cpu() -> Option<(i64, i64)> {
        None
    }
    pub fn disk(_path: &str) -> Option<(i64, i64)> {
        None
    }
    pub fn memory() -> Option<(f64, f64)> {
        None
    }
    pub fn os_theme() -> Option<bool> {
        None
    }
    pub fn os_stamp() -> super::String {
        super::String::new()
    }
}

pub use sys::{cpu, disk, memory, os_stamp, os_theme};
