use super::Claim;
use std::io::{IsTerminal, Read, Write};
use std::string::String;
use std::vec::Vec;

pub const SEP: char = '/';
pub const PRIVATE_MODE: u32 = 0o600;
const MS_PER_SECOND: f64 = 1000.0;
const LOCK_POLL_MS: u32 = 1;

pub fn args() -> Vec<String> {
    std::env::args_os()
        .map(|a| a.to_string_lossy().into_owned())
        .collect()
}

pub fn env(name: &str) -> Option<String> {
    std::env::var_os(name)
        .map(|v| v.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
}

#[cfg(any(target_os = "linux", target_os = "android"))]
const O_NONBLOCK: i32 = 0o4000;
#[cfg(not(any(target_os = "linux", target_os = "android")))]
const O_NONBLOCK: i32 = 0x0004;

pub fn read_limited(path: &str, limit: usize) -> Option<Vec<u8>> {
    use std::os::unix::fs::OpenOptionsExt;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(O_NONBLOCK)
        .open(path)
        .ok()?;
    if !file.metadata().ok()?.is_file() {
        return None;
    }
    let mut data = Vec::new();
    file.take(limit as u64 + 1).read_to_end(&mut data).ok()?;
    (data.len() <= limit).then_some(data)
}

pub fn write(path: &str, data: &[u8]) -> bool {
    std::fs::write(path, data).is_ok()
}

fn resolved(path: &str) -> String {
    let linked = std::fs::symlink_metadata(path).is_ok_and(|m| m.is_symlink());
    let target = linked.then(|| std::fs::canonicalize(path).ok()).flatten();
    target
        .and_then(|t| t.into_os_string().into_string().ok())
        .unwrap_or_else(|| String::from(path))
}

pub fn write_atomic(path: &str, data: &[u8]) -> bool {
    let target = resolved(path);
    let tmp = format!("{}.{}.tmp", target, pid());
    let permissions = std::fs::metadata(&target).ok().map(|m| m.permissions());
    let _ = std::fs::remove_file(&tmp);
    let swapped = create_private(&tmp, data)
        && permissions.is_none_or(|p| std::fs::set_permissions(&tmp, p).is_ok())
        && std::fs::rename(&tmp, &target).is_ok();
    if !swapped {
        let _ = std::fs::remove_file(&tmp);
    }
    swapped
}

pub fn pid() -> u32 {
    std::process::id()
}

pub fn remove(path: &str) -> bool {
    std::fs::remove_file(path).is_ok()
}

pub fn rename(from: &str, to: &str) -> bool {
    std::fs::rename(from, to).is_ok()
}

pub fn create_private(path: &str, data: &[u8]) -> bool {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(PRIVATE_MODE)
        .open(path)
        .and_then(|mut file| file.write_all(data))
        .is_ok()
}

pub struct Lock {
    _file: std::fs::File,
}

pub fn lock(path: &str, wait_ms: u32) -> Claim {
    use std::os::unix::fs::OpenOptionsExt;
    let Ok(file) = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .mode(PRIVATE_MODE)
        .open(path)
    else {
        return Claim::Unavailable;
    };
    let deadline = ticks() + f64::from(wait_ms) / MS_PER_SECOND;
    loop {
        match file.try_lock() {
            Ok(()) => return Claim::Held(Lock { _file: file }),
            Err(std::fs::TryLockError::WouldBlock) if ticks() < deadline => {
                super::sleep(LOCK_POLL_MS)
            }
            Err(std::fs::TryLockError::WouldBlock) => return Claim::Busy,
            Err(_) => return Claim::Unavailable,
        }
    }
}

pub fn make_executable(path: &str) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).is_ok()
}

pub fn process_id() -> u32 {
    std::process::id()
}

const CALL_POLL_MS: u32 = 50;

pub fn call_to(exe: &str, args: &[&str], dir: &str, secs: u32, output: &str) -> Option<i32> {
    use std::process::{Command, Stdio};
    let file = std::fs::File::create(output).ok()?;
    let mut child = Command::new(exe)
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(file)
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let deadline = ticks() + f64::from(secs);
    loop {
        if let Some(status) = child.try_wait().ok()? {
            return status.code();
        }
        if ticks() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        super::sleep(CALL_POLL_MS);
    }
}

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
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    rd.filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with(prefix))
        .collect()
}

pub fn stdin() -> Option<String> {
    let mut input = std::io::stdin();
    if input.is_terminal() {
        return None;
    }
    let mut data = Vec::new();
    let _ = input.read_to_end(&mut data);
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
    std::env::current_exe()
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

pub fn cwd() -> Option<String> {
    std::env::current_dir()
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

pub fn now() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

pub fn ticks() -> f64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_secs_f64()
}

pub fn home() -> Option<String> {
    env("HOME").or_else(|| {
        std::env::home_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .filter(|h| !h.is_empty())
    })
}

pub fn state_dir() -> Option<String> {
    if cfg!(target_os = "macos") {
        return home()
            .map(|h| super::join(&h, &["Library", "Application Support", "claude-statusline"]));
    }
    let base =
        env("XDG_STATE_HOME").or_else(|| home().map(|h| super::join(&h, &[".local", "state"])))?;
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
    use crate::os::linux::disks::{self, Device};
    use crate::os::linux::mounts;
    use crate::os::{Activity, CPU_SOURCE};

    const MOUNT_TABLE: &str = "/proc/self/mountinfo";

    fn read_str(p: &str) -> Option<String> {
        std::fs::read_to_string(p).ok()
    }

    const FIELDS_BEFORE_GUEST: usize = 8;
    const IDLE: usize = 3;
    const IOWAIT: usize = 4;

    pub fn cpu() -> Option<Activity> {
        let s = read_str("/proc/stat")?;
        let line = s.lines().next()?;
        let mut it = line.split_whitespace();
        if it.next()? != "cpu" {
            return None;
        }
        let jiffies: Vec<i64> = it
            .take(FIELDS_BEFORE_GUEST)
            .filter_map(|x| x.parse().ok())
            .collect();
        if jiffies.len() <= IOWAIT {
            return None;
        }
        let clock: i64 = jiffies.iter().sum();
        Some(Activity {
            source: CPU_SOURCE,
            clock,
            busy: clock - jiffies[IDLE] - jiffies[IOWAIT],
        })
    }

    const ANONYMOUS_MAJOR: u64 = 0;

    fn split_device(number: u64) -> Device {
        let major = ((number >> 8) & 0xfff) | ((number >> 32) & 0xffff_f000);
        let minor = (number & 0xff) | ((number >> 12) & 0xffff_ff00);
        (major, minor)
    }

    fn mounted_device(path: &str) -> Option<Device> {
        use std::os::unix::fs::{FileTypeExt, MetadataExt};
        let device = split_device(std::fs::metadata(path).ok()?.dev());
        if device.0 != ANONYMOUS_MAJOR {
            return Some(device);
        }
        let folder = std::fs::canonicalize(path).ok()?;
        let table = read_str(MOUNT_TABLE)?;
        let source = mounts::source(&table, folder.to_str()?)?;
        let node = std::fs::metadata(source).ok()?;
        node.file_type()
            .is_block_device()
            .then(|| split_device(node.rdev()))
    }

    pub fn on_network(path: &str) -> bool {
        read_str(MOUNT_TABLE).is_some_and(|table| mounts::on_network(&table, path))
    }

    fn physical(name: &str) -> bool {
        std::path::Path::new(&format!("/sys/block/{}/device", name)).exists()
    }

    pub fn disk(path: &str) -> Option<Activity> {
        let up = read_str("/proc/uptime")?;
        let uptime_secs: f64 = up.split_whitespace().next()?.parse().ok()?;
        let stats = read_str("/proc/diskstats")?;
        let device = mounted_device(path).or_else(|| disks::sole_disk(&stats, &physical))?;
        let io_ms = disks::io_ms(&stats, device)?;
        Some(Activity {
            source: (device.0 << 32) | device.1,
            clock: (uptime_secs * 1000.0) as i64,
            busy: io_ms,
        })
    }

    pub fn memory() -> Option<(f64, f64)> {
        let host = crate::os::linux::meminfo::parse(&read_str("/proc/meminfo")?)?;
        Some(crate::os::linux::cgroup::narrow(host, &read_str))
    }

    const GSETTINGS_LIMIT: std::time::Duration = std::time::Duration::from_secs(1);

    fn gsettings(key: &str) -> Option<String> {
        use std::process::{Command, Stdio};
        let mut child = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", key])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        let status = crate::os::finish(&mut child, GSETTINGS_LIMIT)?;
        if !status.success() {
            return None;
        }
        let mut text = Vec::new();
        child.stdout.take()?.read_to_end(&mut text).ok()?;
        Some(
            String::from_utf8_lossy(&text)
                .trim()
                .trim_matches('\'')
                .to_lowercase(),
        )
    }

    fn kde() -> Option<bool> {
        let desk = env("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .to_uppercase();
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
                    let channels: Vec<f64> =
                        v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                    if channels.len() >= 3 {
                        return Some(crate::theme::luminance_light([
                            channels[0],
                            channels[1],
                            channels[2],
                        ]));
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
        let config_dir = config_home();
        format!(
            "{}|{}|{}|{}",
            env("GTK_THEME").unwrap_or_default(),
            env("XDG_CURRENT_DESKTOP").unwrap_or_default(),
            stamp_text(
                config_dir
                    .as_ref()
                    .map(|c| super::super::join(c, &["dconf", "user"]))
            ),
            stamp_text(
                config_dir
                    .as_ref()
                    .map(|c| super::super::join(c, &["kdeglobals"]))
            ),
        )
    }
}

#[cfg(target_os = "macos")]
mod sys {
    use super::*;
    use crate::os::{Activity, CPU_SOURCE};
    use std::ffi::c_void;

    type Port = u32;

    extern "C" {
        fn mach_host_self() -> Port;
        fn host_statistics(host: Port, flavor: i32, info: *mut i32, count: *mut u32) -> i32;
        fn host_statistics64(host: Port, flavor: i32, info: *mut i32, count: *mut u32) -> i32;
        fn host_page_size(host: Port, size: *mut usize) -> i32;
        fn sysctlbyname(
            name: *const u8,
            old: *mut c_void,
            oldlen: *mut usize,
            new: *mut c_void,
            newlen: usize,
        ) -> i32;
        fn clock_gettime_nsec_np(clock: u32) -> u64;
        #[cfg_attr(target_arch = "x86_64", link_name = "getfsstat$INODE64")]
        fn getfsstat(entries: *mut u64, size: i32, mode: i32) -> i32;
    }

    #[link(name = "IOKit", kind = "framework")]
    extern "C" {
        fn IOServiceMatching(name: *const u8) -> *mut c_void;
        fn IOServiceGetMatchingServices(main: Port, matching: *mut c_void, iter: *mut u32) -> i32;
        fn IOIteratorNext(iter: u32) -> u32;
        fn IORegistryEntryCreateCFProperty(
            entry: u32,
            key: *const c_void,
            alloc: *const c_void,
            options: u32,
        ) -> *const c_void;
        fn IOObjectRelease(obj: u32) -> i32;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFStringCreateWithCString(alloc: *const c_void, s: *const u8, enc: u32)
            -> *const c_void;
        fn CFDictionaryGetValue(d: *const c_void, key: *const c_void) -> *const c_void;
        fn CFNumberGetValue(n: *const c_void, kind: i32, out: *mut c_void) -> u8;
        fn CFGetTypeID(cf: *const c_void) -> usize;
        fn CFDictionaryGetTypeID() -> usize;
        fn CFNumberGetTypeID() -> usize;
        fn CFRelease(cf: *const c_void);
    }

    const CPU_STATE_IDLE: usize = 2;

    fn host() -> Port {
        static HOST: std::sync::OnceLock<Port> = std::sync::OnceLock::new();
        *HOST.get_or_init(|| unsafe { mach_host_self() })
    }

    pub fn cpu() -> Option<Activity> {
        let mut ticks = [0i32; 4];
        let mut n = 4u32;
        if unsafe { host_statistics(host(), 3, ticks.as_mut_ptr(), &mut n) } != 0 {
            return None;
        }
        let ticks: Vec<i64> = ticks.iter().map(|x| *x as u32 as i64).collect();
        let clock: i64 = ticks.iter().sum();
        Some(Activity {
            source: CPU_SOURCE,
            clock,
            busy: clock - ticks[CPU_STATE_IDLE],
        })
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

    const MNT_NOWAIT: i32 = 2;
    const MNT_LOCAL: u32 = 0x1000;
    const WORD: usize = std::mem::size_of::<u64>();
    const STATFS_SIZE: usize = 2168;
    const STATFS_WORDS: usize = STATFS_SIZE / WORD;
    const FLAGS_AT: usize = 64;
    const FLAGS_LEN: usize = std::mem::size_of::<u32>();
    const MOUNT_POINT_AT: usize = 88;
    const MOUNT_POINT_LEN: usize = 1024;

    fn mount(entry: &[u8]) -> Option<(String, bool)> {
        let flags = entry.get(FLAGS_AT..FLAGS_AT + FLAGS_LEN)?;
        let flags = u32::from_ne_bytes(flags.try_into().ok()?);
        let name = entry.get(MOUNT_POINT_AT..MOUNT_POINT_AT + MOUNT_POINT_LEN)?;
        let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
        let point = String::from_utf8_lossy(&name[..end]).into_owned();
        Some((point, flags & MNT_LOCAL != 0))
    }

    fn mounts() -> Vec<(String, bool)> {
        let counted = unsafe { getfsstat(std::ptr::null_mut(), 0, MNT_NOWAIT) };
        let Ok(count) = usize::try_from(counted) else {
            return Vec::new();
        };
        let mut buffer = vec![0u64; count * STATFS_WORDS];
        let Ok(size) = i32::try_from(buffer.len() * WORD) else {
            return Vec::new();
        };
        let got = unsafe { getfsstat(buffer.as_mut_ptr(), size, MNT_NOWAIT) };
        let Ok(filled) = usize::try_from(got) else {
            return Vec::new();
        };
        let bytes: Vec<u8> = buffer.iter().flat_map(|w| w.to_ne_bytes()).collect();
        bytes
            .as_chunks::<STATFS_SIZE>()
            .0
            .iter()
            .take(filled)
            .filter_map(|entry| mount(entry))
            .collect()
    }

    pub fn on_network(path: &str) -> bool {
        crate::os::deepest_mount(mounts().into_iter(), path).is_some_and(|local| !local)
    }

    const CLOCK_UPTIME_RAW: u32 = 8;

    pub fn disk(_path: &str) -> Option<Activity> {
        let mut iter = 0u32;
        let matching = unsafe { IOServiceMatching(c"IOBlockStorageDriver".as_ptr().cast()) };
        if matching.is_null()
            || unsafe { IOServiceGetMatchingServices(0, matching, &mut iter) } != 0
        {
            return None;
        }
        let k_stats = cfstr(b"Statistics\0");
        let k_read = cfstr(b"Total Time (Read)\0");
        let k_write = cfstr(b"Total Time (Write)\0");
        let mut busy_ns = 0i64;
        let mut drivers = 0u64;
        loop {
            let entry = unsafe { IOIteratorNext(iter) };
            if entry == 0 {
                break;
            }
            let stats =
                unsafe { IORegistryEntryCreateCFProperty(entry, k_stats, std::ptr::null(), 0) };
            if !stats.is_null() {
                if unsafe { CFGetTypeID(stats) == CFDictionaryGetTypeID() } {
                    busy_ns = busy_ns
                        .wrapping_add(number(stats, k_read))
                        .wrapping_add(number(stats, k_write));
                    drivers += 1;
                }
                unsafe { CFRelease(stats) };
            }
            unsafe { IOObjectRelease(entry) };
        }
        unsafe {
            IOObjectRelease(iter);
            CFRelease(k_stats);
            CFRelease(k_read);
            CFRelease(k_write);
        }
        if drivers == 0 {
            return None;
        }
        let uptime_ns = unsafe { clock_gettime_nsec_np(CLOCK_UPTIME_RAW) } as i64;
        Some(Activity {
            source: drivers,
            clock: uptime_ns,
            busy: busy_ns,
        })
    }

    const WIRE_COUNT_OFFSET: usize = 12;
    const PURGEABLE_COUNT_OFFSET: usize = 88;
    const COMPRESSOR_PAGE_COUNT_OFFSET: usize = 128;
    const INTERNAL_PAGE_COUNT_OFFSET: usize = 140;

    pub fn memory() -> Option<(f64, f64)> {
        let mut total = 0u64;
        let mut len = 8usize;
        let r = unsafe {
            sysctlbyname(
                c"hw.memsize".as_ptr().cast(),
                &mut total as *mut u64 as *mut c_void,
                &mut len,
                std::ptr::null_mut(),
                0,
            )
        };
        if r != 0 || total == 0 {
            return None;
        }
        let host = host();
        let mut page = 0usize;
        if unsafe { host_page_size(host, &mut page) } != 0 {
            return None;
        }
        let mut vm_stats = [0i32; 38];
        let mut n = 38u32;
        if unsafe { host_statistics64(host, 4, vm_stats.as_mut_ptr(), &mut n) } != 0 {
            return None;
        }
        let at = |off: usize| vm_stats[off / 4] as u32 as f64;
        let app_pages = at(INTERNAL_PAGE_COUNT_OFFSET) - at(PURGEABLE_COUNT_OFFSET);
        let pages = app_pages + at(WIRE_COUNT_OFFSET) + at(COMPRESSOR_PAGE_COUNT_OFFSET);
        let used = pages * page as f64;
        let used = if used > total as f64 {
            total as f64
        } else {
            used
        };
        Some((total as f64, total as f64 - used))
    }

    fn prefs() -> Option<String> {
        home().map(|h| {
            super::super::join(&h, &["Library", "Preferences", ".GlobalPreferences.plist"])
        })
    }

    fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
        hay.windows(needle.len()).position(|w| w == needle)
    }

    pub fn os_theme() -> Option<bool> {
        let data = super::super::read(&prefs()?)?;
        if data.starts_with(b"bplist") {
            let mut key = vec![0x5F, 0x10, 0x13];
            key.extend_from_slice(b"AppleInterfaceStyle");
            let mut dark = vec![0x54];
            dark.extend_from_slice(b"Dark");
            return Some(!(find(&data, &key).is_some() && find(&data, &dark).is_some()));
        }
        let text = String::from_utf8_lossy(&data);
        let Some(key_at) = text.find("<key>AppleInterfaceStyle</key>") else {
            return Some(true);
        };
        Some(
            !text[key_at..]
                .trim_start_matches("<key>AppleInterfaceStyle</key>")
                .trim_start()
                .starts_with("<string>Dark</string>"),
        )
    }

    pub fn os_stamp() -> String {
        stamp_text(prefs())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        const MACH_PORT_RIGHT_SEND: u32 = 0;
        const KERN_SUCCESS: i32 = 0;
        const READINGS: usize = 100;

        extern "C" {
            static mach_task_self_: Port;
            fn mach_port_get_refs(task: Port, name: Port, right: u32, refs: *mut u32) -> i32;
        }

        fn send_references(port: Port) -> u32 {
            let mut refs = 0u32;
            let task = unsafe { mach_task_self_ };
            let got = unsafe { mach_port_get_refs(task, port, MACH_PORT_RIGHT_SEND, &mut refs) };
            assert_eq!(got, KERN_SUCCESS);
            refs
        }

        #[test]
        fn readings_do_not_pile_up_host_port_references() {
            let port = unsafe { mach_host_self() };
            assert!(cpu().is_some() && memory().is_some());
            let before = send_references(port);
            for _ in 0..READINGS {
                assert!(cpu().is_some());
                assert!(memory().is_some());
            }
            assert_eq!(send_references(port), before);
        }

        #[test]
        fn the_mount_table_holds_the_root_as_a_local_mount() {
            assert!(mounts().contains(&(String::from("/"), true)));
            let temp = std::env::temp_dir();
            assert!(!on_network(&temp.to_string_lossy()));
            assert!(!on_network("/Users"));
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod sys {
    use crate::os::Activity;

    pub fn cpu() -> Option<Activity> {
        None
    }
    pub fn disk(_path: &str) -> Option<Activity> {
        None
    }
    pub fn on_network(_path: &str) -> bool {
        false
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

pub use sys::{cpu, disk, memory, on_network, os_stamp, os_theme};
