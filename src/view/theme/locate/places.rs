use super::wsl;
use crate::os;
use crate::state::State;
use alloc::string::String;

pub struct Side {
    pub home: String,
    pub config: String,
    pub data: String,
}

pub struct Places {
    pub windows: Option<Side>,
    pub unix: Option<Side>,
    pub macos: bool,
    pub unresolved: bool,
    pub ghostty_resources: Option<String>,
}

pub fn var(name: &str) -> Option<String> {
    os::env(name).filter(|v| !v.is_empty())
}

impl Side {
    pub fn windows(home: String) -> Side {
        Side {
            config: os::join(&home, &["AppData", "Roaming"]),
            data: os::join(&home, &["AppData", "Local"]),
            home,
        }
    }
}

impl Places {
    pub fn current(state: &mut State) -> Places {
        let macos = cfg!(target_os = "macos");
        let mut places = Places {
            windows: None,
            unix: None,
            macos,
            unresolved: false,
            ghostty_resources: var("GHOSTTY_RESOURCES_DIR"),
        };
        let home = os::home();
        if cfg!(windows) {
            places.windows = home.map(|home| Side {
                config: var("APPDATA").unwrap_or_else(|| os::join(&home, &["AppData", "Roaming"])),
                data: var("LOCALAPPDATA").unwrap_or_else(|| os::join(&home, &["AppData", "Local"])),
                home,
            });
            return places;
        }
        places.unix = home.map(|home| Side {
            config: var("XDG_CONFIG_HOME").unwrap_or_else(|| os::join(&home, &[".config"])),
            data: if macos {
                os::join(&home, &["Library", "Application Support"])
            } else {
                var("XDG_DATA_HOME").unwrap_or_else(|| os::join(&home, &[".local", "share"]))
            },
            home,
        });
        if var("WSL_DISTRO_NAME").is_some() {
            places.windows = wsl::profile(state).map(Side::windows);
            places.unresolved = places.windows.is_none();
        }
        places
    }
}
