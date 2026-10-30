use std::path::PathBuf;
use std::process::{Command, Stdio};

const INHERITED: [&str; 17] = [
    "CLAUDE_CONFIG_DIR",
    "CLAUDE_CODE_AUTO_COMPACT_WINDOW",
    "CLAUDE_AUTOCOMPACT_PCT_OVERRIDE",
    "STATUSLINE_CONFIG",
    "STATUSLINE_BG",
    "STATUSLINE_LANG",
    "XDG_STATE_HOME",
    "XDG_CONFIG_HOME",
    "XDG_CURRENT_DESKTOP",
    "GTK_THEME",
    "COLORFGBG",
    "TERM_PROGRAM",
    "VSCODE_INJECTION",
    "VSCODE_PID",
    "WT_SESSION",
    "WT_PROFILE_ID",
    "NoDefaultCurrentDirectoryInExePath",
];

pub struct Home(pub PathBuf);

impl Home {
    pub fn new(name: &str) -> Home {
        let root = std::env::temp_dir().join(format!("statusline-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        Home(root)
    }

    pub fn session(&self, account: usize) -> Command {
        self.session_in(&format!(".claude-a{}", account))
    }

    pub fn session_in(&self, folder: &str) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_statusline"));
        for name in INHERITED {
            command.env_remove(name);
        }
        command
            .env("HOME", &self.0)
            .env("USERPROFILE", &self.0)
            .env("LOCALAPPDATA", self.0.join("local"))
            .env("APPDATA", self.0.join("roaming"))
            .env("XDG_STATE_HOME", self.0.join("state"))
            .env("XDG_CONFIG_HOME", self.0.join("config"))
            .env("CLAUDE_CONFIG_DIR", self.0.join(folder))
            .env("NO_COLOR", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
