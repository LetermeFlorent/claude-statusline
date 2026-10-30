use crate::os::*;
use crate::sandbox::Sandbox;

#[test]
fn fail_gives_back_the_exit_code_it_was_given() {
    assert_eq!(fail("", 7), 7);
}

#[test]
fn a_project_nested_past_the_old_path_limit_still_has_its_settings() {
    const OLD_PATH_LIMIT: usize = 260;
    const FOLDER: &str = "a-folder-name-forty-characters-in-length";
    const SETTINGS: &[u8] = b"{\"autoCompactWindow\": 150000}";
    let sandbox = Sandbox::new("long-path");
    let mut project = sandbox.0.clone();
    while project.as_os_str().len() <= OLD_PATH_LIMIT {
        project.push(FOLDER);
    }
    let claude = project.join(".claude");
    std::fs::create_dir_all(&claude).unwrap();
    let folder = claude.to_string_lossy().into_owned();
    let settings = join(&folder, &["settings.json"]);
    std::fs::write(&settings, SETTINGS).unwrap();
    assert!(exists(&settings));
    assert!(stamp(&settings).is_some());
    assert_eq!(read(&settings).as_deref(), Some(SETTINGS));
    assert!(write_atomic(&settings, b"{}"));
    assert_eq!(std::fs::read(&settings).unwrap(), b"{}");
    assert_eq!(list(&folder, "settings"), ["settings.json"]);
    let deeper = join(&folder, &["one", "two"]);
    assert!(mkdirs(&deeper));
    assert!(std::path::Path::new(&deeper).is_dir());
    assert!(remove(&settings));
    assert!(!exists(&settings));
}

#[test]
fn children_start_in_the_given_folder() {
    let sandbox = Sandbox::new("cwd");
    #[cfg(windows)]
    let (shell, script) = (
        join(&system_dir().unwrap(), &["cmd.exe"]),
        ["/c", "cd>here.txt"],
    );
    #[cfg(not(windows))]
    let (shell, script) = (String::from("/bin/sh"), ["-c", "pwd>here.txt"]);
    assert_eq!(call(&shell, &script, &sandbox.dir(), 10), Some(0));
    assert_eq!(sandbox.files(), ["here.txt"]);
}

#[cfg(windows)]
#[test]
fn the_system_folder_holds_cmd() {
    assert!(exists(&join(&system_dir().unwrap(), &["cmd.exe"])));
}
