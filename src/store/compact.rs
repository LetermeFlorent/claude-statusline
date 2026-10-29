use self::cache::Entry;
use self::settings::Setting;
use crate::state::{fingerprint, recent, State};
use alloc::format;
use alloc::string::String;

mod cache;
pub mod files;
mod settings;
mod size;
#[cfg(test)]
mod tests;

const RECHECK_AFTER: f64 = 30.0;

pub struct Disk<'a> {
    pub stat: &'a dyn Fn(&str) -> Option<(u64, i64)>,
    pub read: &'a dyn Fn(&str) -> Option<String>,
}

pub struct Inputs<'a> {
    pub env: Option<&'a str>,
    pub disable: Option<&'a str>,
    pub model: Option<&'a str>,
    pub size: Option<f64>,
}

fn truthy(value: Option<&str>) -> bool {
    value.is_some_and(|v| {
        let v = v.trim();
        !v.is_empty() && v != "0" && !v.eq_ignore_ascii_case("false")
    })
}

fn lookup(
    model: Option<&str>,
    files: &[String; 3],
    state: &mut State,
    now: f64,
    disk: &Disk,
) -> Option<Setting> {
    cache::drop_legacy(state);
    let parts = files.iter().map(String::as_str).chain(model);
    let key = cache::key(&fingerprint(parts));
    let cached = cache::load(state, &key);
    if let Some(entry) = cached
        .as_ref()
        .filter(|entry| recent(entry.at as f64, now, RECHECK_AFTER))
    {
        return entry.setting;
    }
    let mut stamp = String::new();
    for f in files {
        match (disk.stat)(f) {
            Some((size, mtime)) => stamp.push_str(&format!("{}:{};", size, mtime)),
            None => stamp.push_str("0;"),
        }
    }
    let at = now as i64;
    if let Some(entry) = cached.filter(|entry| entry.stamp == stamp) {
        let setting = entry.setting;
        cache::save(state, &key, &Entry { at, ..entry });
        return setting;
    }
    let setting = settings::scan(files, model, disk.read);
    cache::save(state, &key, &Entry { at, stamp, setting });
    setting
}

pub fn window(
    inputs: &Inputs,
    files: &[String; 3],
    state: &mut State,
    now: f64,
    disk: &Disk,
) -> Option<u64> {
    if truthy(inputs.disable) {
        return None;
    }
    let found = lookup(inputs.model, files, state, now, disk);
    let from_env = inputs.env.and_then(size::parse).map(size::clamp);
    let window = match found {
        Some(Setting::Off) => return None,
        Some(Setting::Window(n)) => from_env.or(Some(n)),
        None => from_env,
    }?;
    match inputs.size {
        Some(size) if size >= 1.0 => Some(window.min(size as u64)),
        _ => Some(window),
    }
}
