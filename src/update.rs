use crate::json;
use crate::os;
use crate::update_watch::{asset, marker, newer, parts, STALE_AFTER};
use alloc::format;
use alloc::string::String;

const REPO: &str = "LetermeFlorent/claude-statusline";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const SHOWN_AFTER_SWAP_MS: u32 = 5000;

#[cfg(windows)]
fn curl() -> String {
    let system = os::env("SystemRoot").map(|r| os::join(&r, &["System32", "curl.exe"]));
    system.filter(|p| os::exists(p)).unwrap_or_else(|| String::from("curl.exe"))
}

#[cfg(not(windows))]
fn curl() -> String {
    String::from("curl")
}

fn fetch(url: &str, to: &str, secs: u32) -> bool {
    let limit = format!("{}", secs);
    os::call(&curl(), &["-fsSL", "--max-time", &limit, "-o", to, url], secs + 15) == Some(0)
}

fn latest(dir: &str) -> Option<String> {
    let file = os::join(dir, &["release.json"]);
    let ok = fetch(&format!("https://api.github.com/repos/{}/releases/latest", REPO), &file, 30);
    let text = os::read_text(&file);
    os::remove(&file);
    let release = json::parse(&text.filter(|_| ok)?, false)?;
    let tag = release.get("tag_name")?.str()?;
    parts(tag).map(|_| String::from(tag))
}

fn staging(exe: &str) -> String {
    let n = exe.len();
    if n > 4 && exe.is_char_boundary(n - 4) && exe[n - 4..].eq_ignore_ascii_case(".exe") {
        format!("{}.new.exe", &exe[..n - 4])
    } else {
        format!("{}.new", exe)
    }
}

fn retry(step: &dyn Fn() -> bool) -> bool {
    for _ in 0..10 {
        if step() {
            return true;
        }
        os::sleep(300);
    }
    false
}

fn swap(exe: &str, fresh: &str) -> bool {
    if !cfg!(windows) {
        return os::rename(fresh, exe);
    }
    let old = format!("{}.old-{}", exe, os::now() as i64);
    if !retry(&|| os::rename(exe, &old)) {
        return false;
    }
    if retry(&|| os::rename(fresh, exe)) {
        return true;
    }
    os::rename(&old, exe);
    false
}

fn clean(exe: &str) {
    let (Some(dir), Some(name)) = (os::parent(exe), exe.rsplit(['\\', '/']).next()) else { return };
    for f in os::list(dir, &format!("{}.old", name)) {
        os::remove(&os::join(dir, &[&f]));
    }
}

fn fail(msg: &str) -> i32 {
    os::err(msg);
    1
}

fn install(exe: &str, tag: &str) -> i32 {
    let Some(name) = asset() else { return fail("aucun binaire publie pour ce systeme\n") };
    let fresh = staging(exe);
    let url = format!("https://github.com/{}/releases/download/{}/{}", REPO, tag, name);
    let ready = fetch(&url, &fresh, 300) && os::make_executable(&fresh) && os::call(&fresh, &["--version"], 30) == Some(0);
    if !ready || !swap(exe, &fresh) {
        os::remove(&fresh);
        return fail(&format!("mise a jour vers {} impossible\n", tag));
    }
    os::out(&format!("mise a jour : {} -> {}\n", VERSION, tag));
    os::sleep(SHOWN_AFTER_SWAP_MS);
    0
}

fn attempt(dir: &str) -> i32 {
    let Some(exe) = os::exe() else { return fail("chemin du binaire introuvable\n") };
    clean(&exe);
    let Some(tag) = latest(dir) else { return fail("derniere version introuvable\n") };
    if !newer(&tag, VERSION) {
        os::out(&format!("deja a jour : {}\n", VERSION));
        return 0;
    }
    if let Some(m) = marker() {
        os::write(&m, tag.as_bytes());
    }
    install(&exe, &tag)
}

fn acquire(lock: &str) -> bool {
    if os::create_new(lock) {
        return true;
    }
    let stale = os::stamp(lock).is_none_or(|(_, at)| os::now() - (at as f64) > STALE_AFTER);
    stale && os::remove(lock) && os::create_new(lock)
}

pub fn run() -> i32 {
    let Some(dir) = os::state_dir() else { return fail("dossier d'etat introuvable\n") };
    os::mkdirs(&dir);
    let lock = os::join(&dir, &["update.lock"]);
    if !acquire(&lock) {
        os::out("mise a jour deja en cours\n");
        return 0;
    }
    let code = attempt(&dir);
    if let Some(m) = marker() {
        os::remove(&m);
    }
    os::remove(&lock);
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noms() {
        assert_eq!(staging("C:\\a\\statusline.exe"), "C:\\a\\statusline.new.exe");
        assert_eq!(staging("C:\\a\\STATUSLINE.EXE"), "C:\\a\\STATUSLINE.new.exe");
        assert_eq!(staging("/a/statusline"), "/a/statusline.new");
    }
}
