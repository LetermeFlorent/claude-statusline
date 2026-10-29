use self::settings::Setting;
use crate::state::{fingerprint, recent, State};
use alloc::format;
use alloc::string::String;

mod settings;
mod size;
#[cfg(test)]
mod tests;

const RECHECK_AFTER: f64 = 30.0;

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
    stat: &dyn Fn(&str) -> Option<(u64, i64)>,
    read: &dyn Fn(&str) -> Option<String>,
) -> Option<Setting> {
    let cached = state.get("compact_window").and_then(Setting::load);
    let parts = files.iter().map(String::as_str).chain(model);
    let files_key = fingerprint(parts);
    let same_files = state.get("compact_key") == Some(files_key.as_str());
    let checked = state.num("compact_at");
    if same_files && checked.is_some_and(|at| recent(at, now, RECHECK_AFTER)) {
        return cached;
    }
    let mut stamp = String::new();
    for f in files {
        match stat(f) {
            Some((size, mtime)) => stamp.push_str(&format!("{}:{};", size, mtime)),
            None => stamp.push_str("0;"),
        }
    }
    let now_s = format!("{}", now as i64);
    if same_files && state.get("compact_stamp") == Some(stamp.as_str()) {
        state.set("compact_at", &now_s);
        return cached;
    }
    let found = settings::scan(files, model, read);
    state.set("compact_key", &files_key);
    state.set("compact_stamp", &stamp);
    state.set("compact_at", &now_s);
    match found {
        Some(setting) => state.set("compact_window", &setting.store()),
        None => state.remove("compact_window"),
    }
    found
}

pub fn window(
    inputs: &Inputs,
    files: &[String; 3],
    state: &mut State,
    now: f64,
    stat: &dyn Fn(&str) -> Option<(u64, i64)>,
    read: &dyn Fn(&str) -> Option<String>,
) -> Option<u64> {
    if truthy(inputs.disable) {
        return None;
    }
    let found = lookup(inputs.model, files, state, now, stat, read);
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
