use crate::json;
use crate::os;
use crate::update_watch::{asset, newer, parts, MARKER, STALE_AFTER};
use alloc::format;
use alloc::string::String;

const REPO: &str = "LetermeFlorent/claude-statusline";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const SHOWN_AFTER_SWAP_MS: u32 = 5000;

type Fetch<'a> = &'a dyn Fn(&str, &str, u32) -> bool;

#[derive(Debug, PartialEq)]
enum Outcome {
    UpToDate,
    Updated(String),
}

#[cfg(windows)]
fn curl() -> String {
    let system = os::env("SystemRoot").map(|r| os::join(&r, &["System32", "curl.exe"]));
    system
        .filter(|p| os::exists(p))
        .unwrap_or_else(|| String::from("curl.exe"))
}

#[cfg(not(windows))]
fn curl() -> String {
    String::from("curl")
}

fn download(url: &str, to: &str, secs: u32) -> bool {
    let limit = format!("{}", secs);
    os::call(
        &curl(),
        &["-fsSL", "--max-time", &limit, "-o", to, url],
        secs + 15,
    ) == Some(0)
}

fn latest(dir: &str, fetch: Fetch) -> Option<String> {
    let file = os::join(dir, &["release.json"]);
    let ok = fetch(
        &format!("https://api.github.com/repos/{}/releases/latest", REPO),
        &file,
        30,
    );
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
    let (Some(dir), Some(name)) = (os::parent(exe), exe.rsplit(['\\', '/']).next()) else {
        return;
    };
    for f in os::list(dir, &format!("{}.old", name)) {
        os::remove(&os::join(dir, &[&f]));
    }
}

fn fail(msg: &str) -> i32 {
    os::err(msg);
    1
}

fn install(exe: &str, tag: &str, fetch: Fetch) -> Result<(), String> {
    let Some(name) = asset() else {
        return Err(String::from(
            "aucun binaire publie pour ce systeme
",
        ));
    };
    let fresh = staging(exe);
    let url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        REPO, tag, name
    );
    let ready = fetch(&url, &fresh, 300)
        && os::make_executable(&fresh)
        && os::call(&fresh, &["--version"], 30) == Some(0);
    if !ready || !swap(exe, &fresh) {
        os::remove(&fresh);
        return Err(format!(
            "mise a jour vers {} impossible
",
            tag
        ));
    }
    Ok(())
}

fn attempt(dir: &str, exe: &str, fetch: Fetch) -> Result<Outcome, String> {
    clean(exe);
    let Some(tag) = latest(dir, fetch) else {
        return Err(String::from(
            "derniere version introuvable
",
        ));
    };
    if !newer(&tag, VERSION) {
        return Ok(Outcome::UpToDate);
    }
    os::write(&os::join(dir, &[MARKER]), tag.as_bytes());
    install(exe, &tag, fetch)?;
    Ok(Outcome::Updated(tag))
}

fn report(result: Result<Outcome, String>) -> i32 {
    match result {
        Ok(Outcome::UpToDate) => {
            os::out(&format!(
                "deja a jour : {}
",
                VERSION
            ));
            0
        }
        Ok(Outcome::Updated(tag)) => {
            os::out(&format!(
                "mise a jour : {} -> {}
",
                VERSION, tag
            ));
            os::sleep(SHOWN_AFTER_SWAP_MS);
            0
        }
        Err(msg) => fail(&msg),
    }
}

fn acquire(lock: &str) -> bool {
    if os::create_new(lock) {
        return true;
    }
    let stale = os::stamp(lock).is_none_or(|(_, at)| os::now() - (at as f64) > STALE_AFTER);
    stale && os::remove(lock) && os::create_new(lock)
}

pub fn run() -> i32 {
    let Some(dir) = os::state_dir() else {
        return fail("dossier d'etat introuvable\n");
    };
    os::mkdirs(&dir);
    let lock = os::join(&dir, &["update.lock"]);
    if !acquire(&lock) {
        os::out("mise a jour deja en cours\n");
        return 0;
    }
    let result = match os::exe() {
        Some(exe) => attempt(&dir, &exe, &download),
        None => Err(String::from(
            "chemin du binaire introuvable
",
        )),
    };
    let code = report(result);
    os::remove(&os::join(&dir, &[MARKER]));
    os::remove(&lock);
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noms() {
        assert_eq!(
            staging("C:\\a\\statusline.exe"),
            "C:\\a\\statusline.new.exe"
        );
        assert_eq!(
            staging("C:\\a\\STATUSLINE.EXE"),
            "C:\\a\\STATUSLINE.new.exe"
        );
        assert_eq!(staging("/a/statusline"), "/a/statusline.new");
    }

    use core::cell::RefCell;
    use std::path::PathBuf;

    const OLD_BINARY: &[u8] = b"old";
    const FAILING_BINARY: &[u8] = b"#!/bin/sh\nexit 3\n";
    const EXE_NAME: &str = if cfg!(windows) {
        "statusline.exe"
    } else {
        "statusline"
    };

    struct Sandbox(PathBuf);

    impl Sandbox {
        fn new(name: &str) -> Sandbox {
            let root = std::env::temp_dir().join(format!(
                "statusline-update-{}-{}",
                name,
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&root);
            std::fs::create_dir_all(&root).unwrap();
            Sandbox(root)
        }

        fn dir(&self) -> String {
            self.0.to_string_lossy().into_owned()
        }

        fn files(&self) -> Vec<String> {
            let mut names: Vec<String> = std::fs::read_dir(&self.0)
                .unwrap()
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect();
            names.sort();
            names
        }
    }

    impl Drop for Sandbox {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn working_binary() -> Vec<u8> {
        match std::env::var("SystemRoot") {
            Ok(root) if cfg!(windows) => {
                std::fs::read(format!("{}\\System32\\curl.exe", root)).unwrap()
            }
            _ => b"#!/bin/sh\nexit 0\n".to_vec(),
        }
    }

    struct Release<'a> {
        tag: &'a str,
        binary: &'a [u8],
        asked: RefCell<Vec<String>>,
    }

    impl Release<'_> {
        fn serve(&self, url: &str, to: &str, _: u32) -> bool {
            self.asked.borrow_mut().push(String::from(url));
            let asset_url = format!("/download/{}/{}", self.tag, asset().unwrap());
            let body = if url.ends_with("/releases/latest") {
                format!("{{\"tag_name\": \"{}\"}}", self.tag).into_bytes()
            } else if url.ends_with(&asset_url) {
                self.binary.to_vec()
            } else {
                return false;
            };
            std::fs::write(to, body).is_ok()
        }
    }

    struct Run {
        result: Result<Outcome, String>,
        installed: Vec<u8>,
        files: Vec<String>,
        asked: usize,
    }

    fn update(name: &str, tag: &str, binary: &[u8]) -> Run {
        let sandbox = Sandbox::new(name);
        let exe = sandbox.0.join(EXE_NAME).to_string_lossy().into_owned();
        std::fs::write(&exe, OLD_BINARY).unwrap();
        let release = Release {
            tag,
            binary,
            asked: RefCell::new(Vec::new()),
        };
        let result = attempt(&sandbox.dir(), &exe, &|u, t, s| release.serve(u, t, s));
        Run {
            result,
            installed: std::fs::read(&exe).unwrap(),
            files: sandbox.files(),
            asked: release.asked.into_inner().len(),
        }
    }

    #[test]
    fn newer_tag_is_installed() {
        let binary = working_binary();
        let run = update("newer", "v99.0.0", &binary);
        assert_eq!(run.result, Ok(Outcome::Updated(String::from("v99.0.0"))));
        assert_eq!(run.installed, binary);
        assert!(!run.files.iter().any(|f| f.contains(".new")));
    }

    #[test]
    fn same_tag_is_already_up_to_date() {
        let run = update("same", &format!("v{}", VERSION), &working_binary());
        assert_eq!(run.result, Ok(Outcome::UpToDate));
        assert_eq!(run.installed, OLD_BINARY);
        assert_eq!(run.asked, 1);
        assert_eq!(run.files, [EXE_NAME]);
    }

    #[test]
    fn tag_outside_x_y_z_is_refused() {
        for (i, tag) in ["v99.0", "v99.0.0-rc1", "latest"].iter().enumerate() {
            let run = update(&format!("refused-{}", i), tag, &working_binary());
            assert_eq!(
                run.result,
                Err(String::from("derniere version introuvable\n"))
            );
            assert_eq!(run.installed, OLD_BINARY);
            assert_eq!(run.files, [EXE_NAME]);
        }
    }

    #[test]
    fn binary_failing_its_version_check_is_not_swapped_in() {
        let run = update("failing", "v99.0.0", FAILING_BINARY);
        assert_eq!(
            run.result,
            Err(String::from("mise a jour vers v99.0.0 impossible\n"))
        );
        assert_eq!(run.installed, OLD_BINARY);
        assert_eq!(run.files, [EXE_NAME, MARKER]);
    }
}
