use crate::os::*;
use crate::sandbox::Sandbox;

#[test]
fn files_over_one_mebibyte_are_refused() {
    let sandbox = Sandbox::new("size");
    let path = sandbox.path("settings.json");
    std::fs::write(&path, vec![b' '; MAX_READ_BYTES]).unwrap();
    let at_limit = read(&path).map(|data| data.len());
    std::fs::write(&path, vec![b' '; MAX_READ_BYTES + 1]).unwrap();
    let over_limit = read(&path).map(|data| data.len());
    assert_eq!(at_limit, Some(MAX_READ_BYTES));
    assert_eq!(over_limit, None);
}

#[test]
fn a_larger_limit_reads_larger_files() {
    let sandbox = Sandbox::new("larger");
    let path = sandbox.path(".claude.json");
    std::fs::write(&path, vec![b' '; MAX_READ_BYTES + 1]).unwrap();
    let read_whole = read_limited(&path, 2 * MAX_READ_BYTES).map(|data| data.len());
    assert_eq!(read_whole, Some(MAX_READ_BYTES + 1));
}

const KIB: usize = 1 << 10;

#[test]
fn files_of_every_size_up_to_the_limit_read_whole() {
    let sandbox = Sandbox::new("sizes");
    let path = sandbox.path("state");
    for size in [0, 1, 4 * KIB, 4 * KIB + 1, 64 * KIB + 1, MAX_READ_BYTES] {
        let data: Vec<u8> = (0..size).map(|at| (at % 251) as u8).collect();
        std::fs::write(&path, &data).unwrap();
        assert!(read(&path) == Some(data), "{}", size);
    }
}

#[test]
fn the_read_limit_holds_at_every_size() {
    let sandbox = Sandbox::new("limits");
    let path = sandbox.path("state");
    for limit in [0, 1, 4 * KIB, 64 * KIB] {
        std::fs::write(&path, vec![b'x'; limit]).unwrap();
        let at_limit = read_limited(&path, limit).map(|data| data.len());
        std::fs::write(&path, vec![b'x'; limit + 1]).unwrap();
        assert_eq!(at_limit, Some(limit));
        assert_eq!(read_limited(&path, limit), None, "{}", limit);
    }
}

#[test]
fn a_file_too_large_to_read_is_not_a_missing_file() {
    let sandbox = Sandbox::new("contents");
    let path = sandbox.path("settings.json");
    assert_eq!(read_checked(&path, MAX_READ_BYTES), Contents::Missing);
    std::fs::write(&path, b"{}").unwrap();
    assert_eq!(read_checked(&path, 1), Contents::Unreadable);
    assert_eq!(read_checked(&path, 2), Contents::Bytes(b"{}".to_vec()));
}

#[test]
fn devices_are_refused() {
    let device = if cfg!(windows) { "NUL" } else { "/dev/null" };
    assert_eq!(read(device), None);
}

#[cfg(not(windows))]
#[test]
fn fifo_is_refused_at_once() {
    let sandbox = Sandbox::new("fifo");
    let path = sandbox.path("settings.local.json");
    let made = std::process::Command::new("mkfifo").arg(&path).status();
    assert!(made.is_ok_and(|status| status.success()));
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(read(&path));
    });
    let result = receiver.recv_timeout(std::time::Duration::from_secs(2));
    assert_eq!(result, Ok(None));
}
