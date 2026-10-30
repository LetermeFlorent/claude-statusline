use super::places::{var, Places};
use crate::os;
use alloc::string::String;
use alloc::vec::Vec;

const PRODUCTS: [(&str, &str); 5] = [
    ("Code", ".vscode"),
    ("Code - Insiders", ".vscode-insiders"),
    ("VSCodium", ".vscode-oss"),
    ("Cursor", ".cursor"),
    ("Windsurf", ".windsurf"),
];
const LAUNCHER_WORDS: [(&str, usize); 4] = [
    ("cursor", 3),
    ("windsurf", 4),
    ("vscodium", 2),
    ("insiders", 1),
];
const LAUNCHER_VARIABLES: [&str; 3] = [
    "VSCODE_GIT_ASKPASS_NODE",
    "VSCODE_GIT_ASKPASS_MAIN",
    "VSCODE_CWD",
];
const GHOSTTY_NAMES: [&str; 2] = ["config.ghostty", "config"];

pub fn product_order() -> Vec<usize> {
    let hint = LAUNCHER_VARIABLES
        .iter()
        .filter_map(|name| var(name))
        .map(|value| value.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    let first = LAUNCHER_WORDS
        .iter()
        .find(|(word, _)| hint.contains(word))
        .map_or(0, |(_, index)| *index);
    let mut order = alloc::vec![first];
    order.extend((0..PRODUCTS.len()).filter(|index| *index != first));
    order
}

impl Places {
    pub fn vscode_files(&self, order: &[usize]) -> Vec<String> {
        let windows = self.windows.iter().map(|side| side.config.as_str());
        let unix = self.unix.iter().map(|side| {
            if self.macos {
                side.data.as_str()
            } else {
                side.config.as_str()
            }
        });
        windows
            .chain(unix)
            .flat_map(|base| {
                order
                    .iter()
                    .map(move |i| os::join(base, &[PRODUCTS[*i].0, "User", "settings.json"]))
            })
            .collect()
    }

    pub fn vscode_extensions(&self, order: &[usize]) -> Vec<String> {
        self.windows
            .iter()
            .chain(&self.unix)
            .flat_map(|side| {
                order
                    .iter()
                    .map(move |i| os::join(&side.home, &[PRODUCTS[*i].1, "extensions"]))
            })
            .collect()
    }

    pub fn wt_files(&self) -> Vec<String> {
        let Some(side) = &self.windows else {
            return Vec::new();
        };
        let packages = os::join(&side.data, &["Packages"]);
        let mut names = os::list(&packages, "Microsoft.WindowsTerminal");
        names.sort_by_key(|name| (name.len(), name.clone()));
        let mut out: Vec<String> = names
            .iter()
            .map(|name| os::join(&packages, &[name, "LocalState", "settings.json"]))
            .collect();
        out.push(os::join(
            &side.data,
            &["Microsoft", "Windows Terminal", "settings.json"],
        ));
        out
    }

    pub fn zed_files(&self) -> Vec<String> {
        let windows = self
            .windows
            .iter()
            .map(|side| os::join(&side.config, &["Zed", "settings.json"]));
        let unix = self
            .unix
            .iter()
            .map(|side| os::join(&side.config, &["zed", "settings.json"]));
        windows.chain(unix).collect()
    }

    pub fn zed_theme_dirs(&self) -> Vec<String> {
        let mut dirs = Vec::new();
        let mut installed = Vec::new();
        if let Some(side) = &self.windows {
            dirs.push(os::join(&side.config, &["Zed", "themes"]));
            installed.push(os::join(&side.data, &["Zed", "extensions", "installed"]));
        }
        if let Some(side) = &self.unix {
            dirs.push(os::join(&side.config, &["zed", "themes"]));
            let folder = if self.macos { "Zed" } else { "zed" };
            installed.push(os::join(&side.data, &[folder, "extensions", "installed"]));
        }
        for root in installed {
            for name in os::list(&root, "") {
                dirs.push(os::join(&root, &[&name, "themes"]));
            }
        }
        dirs
    }

    pub fn ghostty_files(&self) -> Vec<String> {
        let Some(side) = &self.unix else {
            return Vec::new();
        };
        let mut folders = alloc::vec![os::join(&side.config, &["ghostty"])];
        if self.macos {
            folders.push(os::join(&side.data, &["com.mitchellh.ghostty"]));
        }
        folders
            .iter()
            .flat_map(|folder| GHOSTTY_NAMES.map(|name| os::join(folder, &[name])))
            .collect()
    }

    pub fn ghostty_theme_dirs(&self) -> Vec<String> {
        let user = self
            .unix
            .iter()
            .map(|side| os::join(&side.config, &["ghostty", "themes"]));
        let shipped = self
            .ghostty_resources
            .iter()
            .map(|resources| os::join(resources, &["themes"]));
        user.chain(shipped).collect()
    }
}
