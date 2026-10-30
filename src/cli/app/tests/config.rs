use crate::app::paths::config_at;
use crate::os::MAX_READ_BYTES;
use crate::sandbox::Sandbox;

#[test]
fn a_config_file_that_cannot_be_read_turns_automatic_updates_off() {
    let sandbox = Sandbox::new("config-auto-update");
    let path = sandbox.path("statusline.json");
    assert!(config_at(None).auto_update);
    assert!(config_at(Some(path.as_str())).auto_update);
    std::fs::write(&path, "{\"auto_update\": true}").unwrap();
    assert!(config_at(Some(path.as_str())).auto_update);
    std::fs::write(&path, "{\"auto_update\": false,").unwrap();
    assert!(!config_at(Some(path.as_str())).auto_update);
    let padding = " ".repeat(MAX_READ_BYTES);
    std::fs::write(&path, format!("{{\"auto_update\": true}}{padding}")).unwrap();
    assert!(!config_at(Some(path.as_str())).auto_update);
}
