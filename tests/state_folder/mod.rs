use crate::common::Home;
use std::path::PathBuf;

impl Home {
    pub fn state_dir(&self) -> PathBuf {
        let base = if cfg!(windows) {
            self.0.join("local")
        } else if cfg!(target_os = "macos") {
            self.0.join("Library").join("Application Support")
        } else {
            self.0.join("state")
        };
        base.join("claude-statusline")
    }
}
