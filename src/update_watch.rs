use crate::os;
use crate::portable;
use crate::state::State;
use alloc::format;
use alloc::string::String;

pub const STALE_AFTER: f64 = 900.0;
pub const MARKER: &str = "updating";
const CHECKED_KEY: &str = "update_at";
const CHECK_EVERY: f64 = 21600.0;

pub fn asset() -> Option<&'static str> {
    if cfg!(all(windows, target_arch = "x86_64")) {
        Some("statusline-windows-x86_64.exe")
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Some("statusline-linux-x86_64")
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        Some("statusline-linux-arm64")
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Some("statusline-macos-arm64")
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        Some("statusline-macos-x86_64")
    } else {
        None
    }
}

pub fn parts(version: &str) -> Option<[u64; 3]> {
    let mut it = version.strip_prefix('v').unwrap_or(version).split('.');
    let mut out = [0u64; 3];
    for slot in out.iter_mut() {
        *slot = it.next()?.parse().ok()?;
    }
    if it.next().is_some() {
        return None;
    }
    Some(out)
}

pub fn newer(remote: &str, local: &str) -> bool {
    matches!((parts(remote), parts(local)), (Some(r), Some(l)) if r > l)
}

pub fn marker() -> Option<String> {
    os::state_dir().map(|d| os::join(&d, &[MARKER]))
}

pub fn busy(now: f64) -> bool {
    marker()
        .and_then(|m| os::stamp(&m))
        .is_some_and(|(_, at)| now - (at as f64) < STALE_AFTER)
}

fn installed(exe: &str) -> bool {
    let rest = portable::home_rest(exe);
    rest.as_deref()
        .and_then(|r| r.rsplit_once('/'))
        .is_some_and(|(dir, _)| portable::same(dir, ".claude/bin"))
}

pub fn due(state: &mut State, now: f64) -> bool {
    if state
        .num(CHECKED_KEY)
        .is_some_and(|at| now >= at && now - at < CHECK_EVERY)
    {
        return false;
    }
    state.set(CHECKED_KEY, &format!("{}", now as i64));
    asset().is_some() && os::exe().is_some_and(|e| installed(&e))
}

pub fn launch() {
    if let Some(exe) = os::exe() {
        os::spawn(&exe, &["--update"]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions() {
        assert!(newer("v2.1.0", "2.0.0"));
        assert!(newer("v2.0.10", "2.0.9"));
        assert!(newer("3.0.0", "2.9.9"));
        assert!(!newer("v2.0.0", "2.0.0"));
        assert!(!newer("v1.9.9", "2.0.0"));
        assert!(!newer("v2.1", "2.0.0"));
        assert!(!newer("v2.1.0-rc1", "2.0.0"));
        assert!(!newer("v2.1.0.1", "2.0.0"));
        assert!(!newer(" v2.1.0", "2.0.0"));
        assert!(!newer("latest", "2.0.0"));
        assert!(asset().is_some());
    }

    #[test]
    fn verification() {
        let mut s = State::parse("");
        let _ = due(&mut s, 1000.0);
        assert_eq!(s.get(CHECKED_KEY), Some("1000"));
        assert!(!due(&mut s, 1000.0 + CHECK_EVERY - 1.0));
        assert_eq!(s.get(CHECKED_KEY), Some("1000"));
        let _ = due(&mut s, 1000.0 + CHECK_EVERY);
        assert_eq!(s.get(CHECKED_KEY), Some("22600"));
        let _ = due(&mut s, 500.0);
        assert_eq!(s.get(CHECKED_KEY), Some("500"));
    }
}
