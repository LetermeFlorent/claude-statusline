use crate::os;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Reverse;

const KEPT_BACKUPS: usize = 3;

fn whole_number(text: &str) -> Option<u64> {
    text.bytes()
        .all(|b| b.is_ascii_digit())
        .then(|| text.parse().ok())?
}

fn stamp(name: &str, prefix: &str) -> Option<(u64, u64)> {
    let rest = name.strip_prefix(prefix)?;
    let (seconds, count) = rest.split_once('-').unwrap_or((rest, "0"));
    Some((whole_number(seconds)?, whole_number(count)?))
}

fn prune(path: &str) {
    let dir = os::parent(path).unwrap_or(".");
    let prefix = format!("{}.bak-", os::file_name(path));
    let mut dated: Vec<((u64, u64), String)> = os::list(dir, &prefix)
        .into_iter()
        .filter_map(|name| Some((stamp(&name, &prefix)?, name)))
        .collect();
    dated.sort_unstable_by_key(|(date, _)| Reverse(*date));
    for (_, name) in dated.into_iter().skip(KEPT_BACKUPS) {
        os::remove(&os::join(dir, &[&name]));
    }
}

pub fn save(path: &str, text: &[u8], now: f64) -> Result<String, String> {
    let base = format!("{}.bak-{}", path, now as i64);
    let mut name = base.clone();
    let mut i = 1;
    while os::exists(&name) {
        name = format!("{}-{}", base, i);
        i += 1;
    }
    if !os::create_private(&name, text) {
        return Err(name);
    }
    prune(path);
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::Sandbox;

    #[test]
    fn backups_keep_the_three_newest() {
        let sandbox = Sandbox::new("backups");
        for name in [
            "settings.json.bak-100",
            "settings.json.bak-200",
            "settings.json.bak-300",
            "settings.json.bak-300-1",
            "settings.json.bak-notes",
            "settings.json.bak-+50",
            "other.json.bak-50",
        ] {
            std::fs::write(sandbox.path(name), b"{}").unwrap();
        }
        let saved = save(&sandbox.path("settings.json"), b"{}", 400.0);
        assert_eq!(saved, Ok(sandbox.path("settings.json.bak-400")));
        assert_eq!(
            sandbox.files(),
            [
                "other.json.bak-50",
                "settings.json.bak-+50",
                "settings.json.bak-300",
                "settings.json.bak-300-1",
                "settings.json.bak-400",
                "settings.json.bak-notes",
            ]
        );
    }

    #[test]
    fn backups_in_the_same_second_are_numbered() {
        let sandbox = Sandbox::new("same-second");
        let path = sandbox.path("settings.json");
        let first = save(&path, b"1", 7.0);
        let second = save(&path, b"2", 7.0);
        assert_eq!(first, Ok(sandbox.path("settings.json.bak-7")));
        assert_eq!(second, Ok(sandbox.path("settings.json.bak-7-1")));
        assert_eq!(
            std::fs::read(sandbox.path("settings.json.bak-7")).unwrap(),
            b"1"
        );
    }

    #[test]
    fn a_failed_backup_names_the_file_it_tried() {
        let sandbox = Sandbox::new("backup-failure");
        let path = os::join(&sandbox.path("missing"), &["settings.json"]);
        assert_eq!(save(&path, b"{}", 7.0), Err(format!("{}.bak-7", path)));
    }

    #[cfg(not(windows))]
    #[test]
    fn backups_are_private() {
        use std::os::unix::fs::PermissionsExt;
        const PERMISSION_BITS: u32 = 0o777;
        let sandbox = Sandbox::new("private");
        let saved = save(&sandbox.path("settings.json"), b"{}", 1.0).unwrap();
        let mode = std::fs::metadata(saved).unwrap().permissions().mode();
        assert_eq!(mode & PERMISSION_BITS, os::PRIVATE_MODE);
    }
}
