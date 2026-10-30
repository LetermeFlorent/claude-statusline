use super::env::pid;
use super::PRIVATE_MODE;
use std::io::{Read, Write};

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

pub fn make_executable(path: &str) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).is_ok()
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
