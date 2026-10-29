use super::ffi::getfsstat;

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

pub fn mounts() -> Vec<(String, bool)> {
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
