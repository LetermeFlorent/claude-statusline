use crate::json::{self, Syntax};
use crate::num;
use crate::state::{fingerprint, State};
use alloc::format;
use alloc::string::String;

const RECHECK_AFTER: f64 = 30.0;

fn positive(s: &str) -> Option<u64> {
    s.trim().parse::<u64>().ok().filter(|n| *n > 0)
}

pub fn window(
    env: Option<&str>,
    files: &[String; 3],
    state: &mut State,
    now: f64,
    stat: &dyn Fn(&str) -> Option<(u64, i64)>,
    read: &dyn Fn(&str) -> Option<String>,
) -> Option<u64> {
    if let Some(n) = env.and_then(positive) {
        return Some(n);
    }
    let cached = state
        .get("compact_window")
        .and_then(|s| s.parse::<u64>().ok())
        .filter(|n| *n > 0);
    let files_key = fingerprint(files.iter().map(String::as_str));
    let same_files = state.get("compact_key") == Some(files_key.as_str());
    if let Some(at) = state.num("compact_at").filter(|_| same_files) {
        if now - at < RECHECK_AFTER {
            return cached;
        }
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
    let mut configured: Option<u64> = None;
    for f in files {
        let Some(settings) = read(f)
            .and_then(|t| json::parse(&t, Syntax::Commented))
            .filter(|v| v.is_obj())
        else {
            continue;
        };
        if settings.get("autoCompactEnabled").and_then(|x| x.bool()) == Some(false) {
            configured = None;
            continue;
        }
        if let Some(n) = settings
            .at(&["env", "CLAUDE_CODE_AUTO_COMPACT_WINDOW"])
            .and_then(|x| x.str())
            .and_then(positive)
        {
            configured = Some(n);
            continue;
        }
        if let Some(n) = settings.get("autoCompactWindow").and_then(|x| x.num()) {
            if n > 0.0 && num::is_integral(n) {
                configured = Some(n as u64);
            }
        }
    }
    state.set("compact_key", &files_key);
    state.set("compact_stamp", &stamp);
    state.set("compact_at", &now_s);
    match configured {
        Some(n) => state.set("compact_window", &format!("{}", n)),
        None => state.remove("compact_window"),
    }
    configured
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files() -> [String; 3] {
        [String::from("a"), String::from("b"), String::from("c")]
    }

    #[test]
    fn reads_compact_window_from_settings() {
        let read = |f: &str| match f {
            "a" => Some(String::from("{\"autoCompactWindow\": 1.5e5}")),
            "b" => Some(String::from(
                "{\"env\": {\"CLAUDE_CODE_AUTO_COMPACT_WINDOW\": \"90000\"}}",
            )),
            _ => None,
        };
        let stat = |f: &str| if f == "c" { None } else { Some((10, 20)) };
        let mut state = State::parse("");
        assert_eq!(
            window(None, &files(), &mut state, 1000.0, &stat, &read),
            Some(90000)
        );
        assert_eq!(state.get("compact_stamp"), Some("10:20;10:20;0;"));
        let no_read = |_: &str| -> Option<String> { panic!() };
        assert_eq!(
            window(None, &files(), &mut state, 1010.0, &stat, &no_read),
            Some(90000)
        );
        assert_eq!(
            window(None, &files(), &mut state, 2000.0, &stat, &no_read),
            Some(90000)
        );
        assert_eq!(
            window(Some(" 5 "), &files(), &mut state, 2000.0, &stat, &no_read),
            Some(5)
        );
        let other_folder = [String::from("a"), String::from("x"), String::from("y")];
        let only_a = |f: &str| if f == "a" { read(f) } else { None };
        assert_eq!(
            window(None, &other_folder, &mut state, 2001.0, &stat, &only_a),
            Some(150000)
        );
        assert_eq!(
            window(None, &files(), &mut state, 2002.0, &stat, &read),
            Some(90000)
        );
        let off = |_: &str| {
            Some(String::from(
                "{\"autoCompactEnabled\": false, \"autoCompactWindow\": 5}",
            ))
        };
        let stat2 = |_: &str| Some((1, 1));
        assert_eq!(
            window(Some("1.5"), &files(), &mut state, 3000.0, &stat2, &off),
            None
        );
        assert_eq!(state.get("compact_window"), None);
    }
}
