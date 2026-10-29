use super::hosts::ghostty::ghostty_keys;
use super::hosts::read_json;
use super::Mode;
use crate::json::Value;
use crate::os;
use alloc::string::String;
use alloc::vec::Vec;

const VSCODE_THEME_KEYS: [&str; 2] = ["id", "label"];

pub fn item_named<'a>(list: &'a Value, keys: &[&str], name: &str) -> Option<&'a Value> {
    list.arr()?.iter().find(|item| {
        keys.iter()
            .any(|key| item.get(key).and_then(|v| v.str()) == Some(name))
    })
}

pub fn vscode_ui_theme(roots: &[String], name: &str) -> Option<Mode> {
    roots.iter().find_map(|root| {
        os::list(root, "").iter().find_map(|folder| {
            let manifest = read_json(&os::join(root, &[folder, "package.json"]))?;
            let themes = manifest.at(&["contributes", "themes"])?;
            match item_named(themes, &VSCODE_THEME_KEYS, name)?
                .get("uiTheme")?
                .str()?
            {
                "vs" | "hc-light" => Some(Mode::Light),
                _ => Some(Mode::Dark),
            }
        })
    })
}

pub fn zed_appearance(dirs: &[String], name: &str) -> Option<Mode> {
    dirs.iter().find_map(|dir| {
        os::list(dir, "")
            .iter()
            .filter(|file| file.ends_with(".json"))
            .find_map(|file| {
                let family = read_json(&os::join(dir, &[file]))?;
                match item_named(family.get("themes")?, &["name"], name)?
                    .get("appearance")?
                    .str()?
                {
                    "light" => Some(Mode::Light),
                    "dark" => Some(Mode::Dark),
                    _ => None,
                }
            })
    })
}

pub fn ghostty_theme(dirs: &[String], name: &str) -> Option<Mode> {
    let paths: Vec<String> = if name.starts_with('/') {
        alloc::vec![String::from(name)]
    } else {
        dirs.iter().map(|dir| os::join(dir, &[name])).collect()
    };
    let text = paths.iter().find_map(|path| os::read_text(path))?;
    ghostty_keys(&text).0
}
