use super::color::color_mode;
use super::named;
use crate::json::Value;
use crate::theme::{Lookup, Mode};
use alloc::format;
use alloc::vec::Vec;

const TERMINAL_BACKGROUND: &str = "terminal.background";

fn flat_or_nested<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    v.get(key).or_else(|| {
        let parts: Vec<&str> = key.split('.').collect();
        v.at(&parts)
    })
}

fn setting<'a>(layers: &'a [Value], key: &str) -> Option<&'a Value> {
    layers.iter().find_map(|layer| flat_or_nested(layer, key))
}

fn terminal_background(layers: &[Value], theme: Option<&str>) -> Option<Mode> {
    let colors = setting(layers, "workbench.colorCustomizations")?;
    theme
        .and_then(|name| colors.get(&format!("[{}]", name)))
        .and_then(|scope| scope.get(TERMINAL_BACKGROUND))
        .or_else(|| colors.get(TERMINAL_BACKGROUND))
        .and_then(|color| color.str())
        .and_then(color_mode)
}

pub fn vscode(layers: &[Value], lookup: &Lookup<'_>) -> Mode {
    let theme = setting(layers, "workbench.colorTheme").and_then(|v| v.str());
    if let Some(mode) = terminal_background(layers, theme) {
        return mode;
    }
    if setting(layers, "window.autoDetectColorScheme").and_then(|v| v.bool()) == Some(true) {
        return Mode::System;
    }
    theme.map_or(Mode::Dark, |name| named(name, lookup))
}
