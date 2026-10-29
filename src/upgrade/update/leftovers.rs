use super::swap::split_extension;
use crate::os;
use alloc::format;

const TEMP_SUFFIX: &str = ".tmp";
const TEMP_KEPT_SECS: i64 = 60;

fn remove_named(dir: &str, prefix: &str) {
    for name in os::list(dir, prefix) {
        if name.starts_with(prefix) {
            os::remove(&os::join(dir, &[&name]));
        }
    }
}

pub fn clean_binaries(exe: &str) {
    let Some(dir) = os::parent(exe) else {
        return;
    };
    let name = os::file_name(exe);
    let (stem, extension) = split_extension(name);
    os::remove(&os::join(dir, &[&format!("{}.new{}", stem, extension)]));
    remove_named(dir, &format!("{}.new-", stem));
    remove_named(dir, &format!("{}.old-", name));
}

pub fn clean_temp(dir: &str, now: f64) {
    for name in os::list(dir, "") {
        if !name.ends_with(TEMP_SUFFIX) {
            continue;
        }
        let path = os::join(dir, &[&name]);
        if os::stamp(&path).is_some_and(|(_, at)| now as i64 - at > TEMP_KEPT_SECS) {
            os::remove(&path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::Sandbox;

    const EXE_NAME: &str = "statusline.exe";
    const HOUR: f64 = 3600.0;

    #[test]
    fn staging_and_old_binaries_are_removed_and_nothing_else() {
        let sandbox = Sandbox::new("leftover-binaries");
        let kept = [EXE_NAME, "statusline.json", "statusline.newer.txt.bak"];
        let removed = [
            "statusline.new.exe",
            "statusline.new-4242.exe",
            "statusline.exe.old-1700000000",
            "statusline.exe.old-4242",
        ];
        for name in kept.iter().chain(&removed) {
            std::fs::write(sandbox.path(name), b"x").unwrap();
        }
        clean_binaries(&sandbox.path(EXE_NAME));
        assert_eq!(sandbox.files(), kept);
    }

    #[test]
    fn a_binary_named_like_a_staging_file_keeps_itself() {
        let sandbox = Sandbox::new("leftover-self");
        let exe = sandbox.path("statusline.new.exe");
        std::fs::write(&exe, b"x").unwrap();
        clean_binaries(&exe);
        assert_eq!(sandbox.files(), ["statusline.new.exe"]);
    }

    #[test]
    fn only_temp_files_older_than_a_minute_are_removed() {
        let sandbox = Sandbox::new("leftover-temp");
        for name in ["state", "state.12.tmp", "update.lock"] {
            std::fs::write(sandbox.path(name), b"x").unwrap();
        }
        clean_temp(&sandbox.dir(), os::now());
        assert_eq!(sandbox.files(), ["state", "state.12.tmp", "update.lock"]);
        clean_temp(&sandbox.dir(), os::now() + HOUR);
        assert_eq!(sandbox.files(), ["state", "update.lock"]);
    }
}
