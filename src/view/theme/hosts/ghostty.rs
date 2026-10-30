use super::color::color_mode;
use super::named;
use crate::theme::{Lookup, Mode};

pub fn ghostty_keys(text: &str) -> (Option<Mode>, Option<&str>) {
    let mut background = None;
    let mut theme = None;
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"');
        match key.trim() {
            "background" if value.is_empty() => background = None,
            "background" => background = color_mode(value).or(background),
            "theme" if value.is_empty() => theme = None,
            "theme" => theme = Some(value),
            _ => {}
        }
    }
    (background, theme)
}

pub fn ghostty(text: &str, lookup: &Lookup<'_>) -> Mode {
    match ghostty_keys(text) {
        (Some(mode), _) => mode,
        (None, Some(pair)) if pair.contains("light:") || pair.contains("dark:") => Mode::System,
        (None, Some(name)) => named(name, lookup),
        (None, None) => Mode::Dark,
    }
}
