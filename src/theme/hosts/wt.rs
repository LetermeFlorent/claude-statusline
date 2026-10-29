use super::color::shade;
use super::read_json;
use crate::config::hex;
use crate::json::Value;
use crate::os;
use crate::theme::{lookup, Mode};
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Reverse;

const WT_LIGHT: [&str; 3] = ["One Half Light", "Solarized Light", "Tango Light"];
const WT_DEFAULT_SCHEME: &str = "Campbell";

fn wt_profiles(settings: &Value) -> (Option<&[Value]>, Option<&Value>) {
    match settings.get("profiles") {
        Some(Value::Arr(list)) => (Some(list.as_slice()), None),
        Some(profiles) if profiles.is_obj() => (
            profiles.get("list").and_then(|list| list.arr()),
            profiles
                .get("defaults")
                .filter(|defaults| defaults.is_obj()),
        ),
        _ => (None, None),
    }
}

fn wt_profile<'a>(settings: &'a Value, id: &str) -> Option<&'a Value> {
    wt_profiles(settings).0?.iter().find(|profile| {
        profile
            .get("guid")
            .and_then(|guid| guid.str())
            .is_some_and(|guid| guid.eq_ignore_ascii_case(id))
    })
}

fn app_theme(settings: &Value) -> Mode {
    let name = settings
        .get("theme")
        .and_then(|t| t.str())
        .unwrap_or("system");
    let custom = settings
        .get("themes")
        .and_then(|themes| lookup::item_named(themes, &["name"], name))
        .and_then(|theme| theme.at(&["window", "applicationTheme"]))
        .and_then(|v| v.str());
    match custom.unwrap_or(name) {
        "light" | "legacyLight" => Mode::Light,
        "dark" | "legacyDark" => Mode::Dark,
        _ => Mode::System,
    }
}

pub fn wt(settings: &Value, profile_id: Option<&str>, os_light: &dyn Fn() -> bool) -> Mode {
    let defaults = wt_profiles(settings).1;
    let wanted = profile_id.or_else(|| settings.get("defaultProfile").and_then(|d| d.str()));
    let profile = wanted.and_then(|id| wt_profile(settings, id));
    let pick = |key: &str| {
        profile
            .and_then(|p| p.get(key))
            .or_else(|| defaults.and_then(|d| d.get(key)))
    };
    if let Some(background) = hex(pick("background")) {
        return shade(background);
    }
    let scheme = match pick("colorScheme") {
        Some(Value::Str(name)) => name.as_str(),
        Some(pair) if pair.is_obj() => {
            let light = match app_theme(settings) {
                Mode::Light => true,
                Mode::Dark => false,
                Mode::System => os_light(),
            };
            let side = if light { "light" } else { "dark" };
            pair.get(side)
                .and_then(|s| s.str())
                .unwrap_or(WT_DEFAULT_SCHEME)
        }
        _ => WT_DEFAULT_SCHEME,
    };
    let custom = settings
        .get("schemes")
        .and_then(|schemes| lookup::item_named(schemes, &["name"], scheme));
    match custom {
        Some(custom) => hex(custom.get("background")).map_or(Mode::Dark, shade),
        None if WT_LIGHT.contains(&scheme) => Mode::Light,
        None => Mode::Dark,
    }
}

pub fn wt_settings(files: &[String], profile_id: Option<&str>) -> Option<Value> {
    let found: Vec<(Value, i64)> = files
        .iter()
        .filter_map(|file| Some((read_json(file)?, os::stamp(file)?.1)))
        .collect();
    let holder = profile_id.and_then(|id| {
        found
            .iter()
            .position(|(settings, _)| wt_profile(settings, id).is_some())
    });
    let newest = found
        .iter()
        .enumerate()
        .max_by_key(|(index, (_, mtime))| (*mtime, Reverse(*index)))
        .map(|(index, _)| index);
    let index = holder.or(newest)?;
    found.into_iter().nth(index).map(|(settings, _)| settings)
}
