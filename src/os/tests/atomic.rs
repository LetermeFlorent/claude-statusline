use crate::os::*;
use crate::sandbox::Sandbox;

#[cfg(not(windows))]
#[test]
fn atomic_write_keeps_the_mode_and_starts_private() {
    use std::os::unix::fs::PermissionsExt;
    const PERMISSION_BITS: u32 = 0o777;
    const SHARED_MODE: u32 = 0o640;
    let sandbox = Sandbox::new("mode");
    let (fresh, shared) = (sandbox.path("fresh.json"), sandbox.path("shared.json"));
    std::fs::write(&shared, b"{}").unwrap();
    std::fs::set_permissions(&shared, std::fs::Permissions::from_mode(SHARED_MODE)).unwrap();
    assert!(write_atomic(&fresh, b"{}"));
    assert!(write_atomic(&shared, b"[]"));
    let mode = |path: &str| std::fs::metadata(path).unwrap().permissions().mode() & PERMISSION_BITS;
    assert_eq!([mode(&fresh), mode(&shared)], [PRIVATE_MODE, SHARED_MODE]);
    assert_eq!(std::fs::read(&shared).unwrap(), b"[]");
}

#[test]
fn atomic_write_clears_a_leftover_temp_file() {
    let sandbox = Sandbox::new("leftover");
    let path = sandbox.path("state.json");
    std::fs::write(format!("{}.{}.tmp", path, std::process::id()), b"{}").unwrap();
    assert!(write_atomic(&path, b"[]"));
    assert_eq!(std::fs::read(&path).unwrap(), b"[]");
    assert_eq!(sandbox.files(), ["state.json"]);
}

#[test]
fn an_open_reader_keeps_its_copy_while_the_file_is_replaced() {
    use std::io::Read;
    let sandbox = Sandbox::new("open-reader");
    let path = sandbox.path("state");
    std::fs::write(&path, b"old").unwrap();
    let mut reader = std::fs::File::open(&path).unwrap();
    assert!(write_atomic(&path, b"new"));
    let mut seen = Vec::new();
    reader.read_to_end(&mut seen).unwrap();
    assert_eq!(seen, b"old");
    assert_eq!(std::fs::read(&path).unwrap(), b"new");
    assert_eq!(sandbox.files(), ["state"]);
}

#[test]
fn a_reader_racing_the_swaps_always_gets_one_whole_version() {
    const SWAPS: usize = 300;
    const VERSIONS: [&[u8]; 2] = [b"q5_pct@a 10\n", b"q5_pct@a 20\nq7_pct@a 30\n"];
    let sandbox = Sandbox::new("racing-reader");
    let path = sandbox.path("state");
    std::fs::write(&path, VERSIONS[0]).unwrap();
    let target = path.clone();
    let writer = std::thread::spawn(move || {
        (0..SWAPS).all(|swap| write_atomic(&target, VERSIONS[swap % 2]))
    });
    while !writer.is_finished() {
        let seen = read(&path);
        let whole = seen.as_deref().is_some_and(|data| VERSIONS.contains(&data));
        assert!(whole, "{:?}", seen);
    }
    assert!(writer.join().unwrap());
    assert_eq!(sandbox.files(), ["state"]);
}

#[cfg(windows)]
#[test]
fn a_file_held_without_delete_sharing_is_left_whole() {
    use std::os::windows::fs::OpenOptionsExt;
    const SHARE_READ_AND_WRITE: u32 = 3;
    let sandbox = Sandbox::new("held");
    let path = sandbox.path("state");
    std::fs::write(&path, b"old").unwrap();
    let _holder = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(SHARE_READ_AND_WRITE)
        .open(&path)
        .unwrap();
    assert!(!write_atomic(&path, b"new"));
    assert_eq!(std::fs::read(&path).unwrap(), b"old");
    assert_eq!(sandbox.files(), ["state"]);
}

#[test]
fn a_temp_file_that_cannot_be_made_leaves_the_file_whole() {
    let sandbox = Sandbox::new("blocked-temp");
    let path = sandbox.path("state");
    std::fs::write(&path, b"old").unwrap();
    std::fs::create_dir(format!("{}.{}.tmp", path, std::process::id())).unwrap();
    assert!(!write_atomic(&path, b"new"));
    assert_eq!(std::fs::read(&path).unwrap(), b"old");
}
