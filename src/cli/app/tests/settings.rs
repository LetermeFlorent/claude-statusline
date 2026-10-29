use crate::app::install::{add_line, remove_line, TEMPLATE};
use crate::check;
use crate::messages;
use crate::sandbox::Sandbox;

#[test]
fn uninstall_names_each_case() {
    let sandbox = Sandbox::new("uninstall-cases");
    let path = sandbox.path("settings.json");
    assert_eq!(remove_line(&path), Ok(messages::NO_SETTINGS.with(&[&path])));
    for text in ["", "{\"model\": \"opus\"}"] {
        std::fs::write(&path, text).unwrap();
        let no_line = Ok(messages::NOTHING_TO_REMOVE.with(&[&path]));
        assert_eq!(remove_line(&path), no_line, "{}", text);
    }
    for raw in [&b"[1]"[..], b"{\"a\": \"\xff\"}"] {
        std::fs::write(&path, raw).unwrap();
        let invalid = Err(messages::SETTINGS_INVALID.with(&[&path]));
        assert_eq!(remove_line(&path), invalid);
        assert_eq!(add_line(&path, "x"), invalid);
        assert_eq!(std::fs::read(&path).unwrap(), raw);
    }
    assert_eq!(sandbox.files(), ["settings.json"]);
}

#[test]
fn unreadable_settings_are_an_error() {
    let sandbox = Sandbox::new("unreadable-settings");
    let path = sandbox.path("settings.json");
    std::fs::create_dir(&path).unwrap();
    let unreadable = Err(messages::SETTINGS_UNREADABLE.with(&[&path]));
    assert_eq!(remove_line(&path), unreadable);
    assert_eq!(add_line(&path, "x"), unreadable);
}

#[test]
fn an_empty_file_counts_as_an_empty_object() {
    let sandbox = Sandbox::new("empty-settings");
    let path = sandbox.path("settings.json");
    std::fs::write(&path, " \n").unwrap();
    assert!(add_line(&path, "x").is_ok());
    let written = std::fs::read_to_string(&path).unwrap();
    assert!(written.contains("\"statusLine\""), "{}", written);
}

#[test]
fn the_printed_backup_is_on_disk() {
    let sandbox = Sandbox::new("printed-backup");
    let path = sandbox.path("settings.json");
    std::fs::write(&path, "{\"statusLine\": {}}").unwrap();
    let told = remove_line(&path).unwrap();
    let files = sandbox.files();
    let backup = files.iter().find(|name| *name != "settings.json").unwrap();
    assert!(backup.starts_with("settings.json.bak-"), "{}", backup);
    let backup_told = messages::BACKUP.with(&[&sandbox.path(backup)]);
    assert_eq!(told, backup_told + &messages::REMOVED.with(&[&path]));
}

#[test]
fn the_written_template_passes_the_check() {
    assert!(check::problems(TEMPLATE).is_empty());
}
