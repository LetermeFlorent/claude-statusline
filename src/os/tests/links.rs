use crate::os::*;
use crate::sandbox::Sandbox;
#[cfg(not(windows))]
use std::os::unix::fs::symlink;
#[cfg(windows)]
use std::os::windows::fs::symlink_file as symlink;

#[test]
#[cfg_attr(windows, ignore = "creating a symbolic link needs an elevated token")]
fn atomic_write_keeps_a_symbolic_link() {
    let sandbox = Sandbox::new("link");
    let (real, link) = (sandbox.path("real.json"), sandbox.path("settings.json"));
    std::fs::write(&real, b"{}").unwrap();
    symlink(&real, &link).unwrap();
    assert!(write_atomic(&link, b"[]"));
    assert!(std::fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(std::fs::read(&real).unwrap(), b"[]");
    assert_eq!(sandbox.files(), ["real.json", "settings.json"]);
}

#[test]
#[cfg_attr(windows, ignore = "creating a symbolic link needs an elevated token")]
fn the_stamp_of_a_link_follows_its_target() {
    let sandbox = Sandbox::new("stamp-link");
    let (real, link) = (sandbox.path("real.json"), sandbox.path("settings.json"));
    std::fs::write(&real, b"{}").unwrap();
    symlink(&real, &link).unwrap();
    let before = stamp(&link);
    std::fs::write(&real, b"{\"theme\": 1}").unwrap();
    assert_ne!(stamp(&link), before);
    assert_eq!(stamp(&link), stamp(&real));
}
