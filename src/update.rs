use crate::json::{self, Syntax};
use crate::os;
use crate::update_watch::{asset, newer, parts, MARKER, STALE_AFTER};
use alloc::format;
use alloc::string::String;

const REPO: &str = "LetermeFlorent/claude-statusline";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const SHOWN_AFTER_SWAP_MS: u32 = 5000;
const HTTP_OK: u16 = 200;
const NO_RELEASE: &str = "derniere version introuvable\n";
const RATE_LIMIT_STATUSES: [u16; 2] = [403, 429];
const HTTPS_ONLY: &str = "=https";
const CURL_MISSING: &str = "curl.exe introuvable dans le dossier systeme\n";
const RATE_LIMITED: &str = "limite de requetes GitHub atteinte, reessayez plus tard\n";

type Fetch<'a> = &'a dyn Fn(&str, &str, u32) -> Option<u16>;

#[derive(Debug, PartialEq)]
enum Outcome {
    UpToDate,
    Updated(String),
}

#[cfg(windows)]
fn curl_in(system: &str) -> Option<String> {
    let path = os::join(system, &["curl.exe"]);
    os::exists(&path).then_some(path)
}

#[cfg(windows)]
fn curl() -> Option<String> {
    curl_in(&os::system_dir()?)
}

#[cfg(not(windows))]
fn curl() -> Option<String> {
    Some(String::from("curl"))
}

fn last_status(headers: &str) -> Option<u16> {
    let line = headers.lines().rfind(|l| l.starts_with("HTTP/"))?;
    line.split_whitespace().nth(1)?.parse().ok()
}

fn curl_args<'a>(limit: &'a str, headers: &'a str, to: &'a str, url: &'a str) -> [&'a str; 13] {
    [
        "-q",
        "-sSL",
        "--proto",
        HTTPS_ONLY,
        "--proto-redir",
        HTTPS_ONLY,
        "--max-time",
        limit,
        "-D",
        headers,
        "-o",
        to,
        url,
    ]
}

struct Curl<'a> {
    program: String,
    dir: &'a str,
}

impl Curl<'_> {
    fn fetch(&self, url: &str, to: &str, secs: u32) -> Option<u16> {
        let headers = format!("{}.headers", to);
        let limit = format!("{}", secs);
        let args = curl_args(&limit, &headers, to, url);
        let finished = os::call(&self.program, &args, self.dir, secs + 15) == Some(0);
        let status = os::read_text(&headers).and_then(|h| last_status(&h));
        os::remove(&headers);
        status.filter(|_| finished)
    }
}

fn latest(dir: &str, fetch: Fetch) -> Result<String, String> {
    let file = os::join(dir, &["release.json"]);
    let url = format!("https://api.github.com/repos/{}/releases/latest", REPO);
    let status = fetch(&url, &file, 30);
    let text = os::read_text(&file);
    os::remove(&file);
    if status.is_some_and(|s| RATE_LIMIT_STATUSES.contains(&s)) {
        return Err(String::from(RATE_LIMITED));
    }
    let tag = text
        .filter(|_| status == Some(HTTP_OK))
        .and_then(|t| json::parse(&t, Syntax::Tolerant))
        .and_then(|release| release.get("tag_name")?.str().map(String::from))
        .filter(|tag| parts(tag).is_some());
    tag.ok_or_else(|| String::from(NO_RELEASE))
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

fn install(dir: &str, exe: &str, tag: &str, fetch: Fetch) -> Result<(), String> {
    let Some(name) = asset() else {
        return Err(String::from("aucun binaire publie pour ce systeme\n"));
    };
    let fresh = staging(exe);
    let url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        REPO, tag, name
    );
    let ready = fetch(&url, &fresh, 300) == Some(HTTP_OK)
        && os::make_executable(&fresh)
        && os::call(&fresh, &["--version"], dir, 30) == Some(0);
    if !ready || !swap(exe, &fresh) {
        os::remove(&fresh);
        return Err(format!("mise a jour vers {} impossible\n", tag));
    }
    Ok(())
}

fn attempt(dir: &str, exe: &str, fetch: Fetch) -> Result<Outcome, String> {
    clean(exe);
    let tag = latest(dir, fetch)?;
    if !newer(&tag, VERSION) {
        return Ok(Outcome::UpToDate);
    }
    os::write(&os::join(dir, &[MARKER]), tag.as_bytes());
    install(dir, exe, &tag, fetch)?;
    Ok(Outcome::Updated(tag))
}

fn report(result: Result<Outcome, String>) -> i32 {
    match result {
        Ok(Outcome::UpToDate) => {
            os::out(&format!("deja a jour : {}\n", VERSION));
            0
        }
        Ok(Outcome::Updated(tag)) => {
            os::out(&format!("mise a jour : {} -> {}\n", VERSION, tag));
            os::sleep(SHOWN_AFTER_SWAP_MS);
            0
        }
        Err(msg) => fail(&msg),
    }
}

fn acquire(lock: &str) -> bool {
    if os::create_private(lock, &[]) {
        return true;
    }
    let stale = os::stamp(lock).is_none_or(|(_, at)| os::now() - (at as f64) > STALE_AFTER);
    stale && os::remove(lock) && os::create_private(lock, &[])
}

fn take_lock(dir: &str) -> Result<Option<String>, String> {
    if !os::mkdirs(dir) {
        return Err(format!("dossier d'etat impossible a creer : {}\n", dir));
    }
    let lock = os::join(dir, &["update.lock"]);
    Ok(acquire(&lock).then_some(lock))
}

pub fn run() -> i32 {
    let Some(dir) = os::state_dir() else {
        return fail("dossier d'etat introuvable\n");
    };
    let Some(program) = curl() else {
        return fail(CURL_MISSING);
    };
    let lock = match take_lock(&dir) {
        Ok(Some(lock)) => lock,
        Ok(None) => {
            os::out("mise a jour deja en cours\n");
            return 0;
        }
        Err(msg) => return fail(&msg),
    };
    let curl = Curl { program, dir: &dir };
    let fetch = |url: &str, to: &str, secs: u32| curl.fetch(url, to, secs);
    let result = match os::exe() {
        Some(exe) => attempt(&dir, &exe, &fetch),
        None => Err(String::from("chemin du binaire introuvable\n")),
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
    fn staging_name_puts_new_before_the_extension() {
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

    use crate::sandbox::Sandbox;
    use core::cell::RefCell;

    const OLD_BINARY: &[u8] = b"old";
    const FAILING_BINARY: &[u8] = b"#!/bin/sh\nexit 3\n";
    const EXE_NAME: &str = if cfg!(windows) {
        "statusline.exe"
    } else {
        "statusline"
    };

    fn working_binary() -> Vec<u8> {
        if cfg!(windows) {
            std::fs::read(curl().unwrap()).unwrap()
        } else {
            b"#!/bin/sh\nexit 0\n".to_vec()
        }
    }

    struct Release<'a> {
        tag: &'a str,
        binary: &'a [u8],
        api_status: u16,
        asked: RefCell<Vec<String>>,
    }

    fn release<'a>(tag: &'a str, binary: &'a [u8]) -> Release<'a> {
        Release {
            tag,
            binary,
            api_status: HTTP_OK,
            asked: RefCell::new(Vec::new()),
        }
    }

    impl Release<'_> {
        fn serve(&self, url: &str, to: &str, _: u32) -> Option<u16> {
            self.asked.borrow_mut().push(String::from(url));
            let asset_url = format!("/download/{}/{}", self.tag, asset().unwrap());
            let (status, body) = if url.ends_with("/releases/latest") && self.api_status != HTTP_OK
            {
                (
                    self.api_status,
                    b"{\"message\": \"API rate limit exceeded\"}".to_vec(),
                )
            } else if url.ends_with("/releases/latest") {
                (
                    HTTP_OK,
                    format!("{{\"tag_name\": \"{}\"}}", self.tag).into_bytes(),
                )
            } else if url.ends_with(&asset_url) {
                (HTTP_OK, self.binary.to_vec())
            } else {
                (404, b"Not Found".to_vec())
            };
            std::fs::write(to, body).ok().map(|_| status)
        }
    }

    struct Run {
        result: Result<Outcome, String>,
        installed: Vec<u8>,
        files: Vec<String>,
        asked: usize,
    }

    fn update(name: &str, release: Release) -> Run {
        let sandbox = Sandbox::new(name);
        let exe = sandbox.path(EXE_NAME);
        std::fs::write(&exe, OLD_BINARY).unwrap();
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
        let run = update("newer", release("v99.0.0", &binary));
        assert_eq!(run.result, Ok(Outcome::Updated(String::from("v99.0.0"))));
        assert_eq!(run.installed, binary);
        assert!(!run.files.iter().any(|f| f.contains(".new")));
    }

    #[test]
    fn same_tag_is_already_up_to_date() {
        let run = update("same", release(&format!("v{}", VERSION), &working_binary()));
        assert_eq!(run.result, Ok(Outcome::UpToDate));
        assert_eq!(run.installed, OLD_BINARY);
        assert_eq!(run.asked, 1);
        assert_eq!(run.files, [EXE_NAME]);
    }

    #[test]
    fn tag_outside_x_y_z_is_refused() {
        for (i, tag) in ["v99.0", "v99.0.0-rc1", "latest"].iter().enumerate() {
            let run = update(&format!("refused-{}", i), release(tag, &working_binary()));
            assert_eq!(run.result, Err(String::from(NO_RELEASE)));
            assert_eq!(run.installed, OLD_BINARY);
            assert_eq!(run.files, [EXE_NAME]);
        }
    }

    #[test]
    fn binary_failing_its_version_check_is_not_swapped_in() {
        let run = update("failing", release("v99.0.0", FAILING_BINARY));
        assert_eq!(
            run.result,
            Err(String::from("mise a jour vers v99.0.0 impossible\n"))
        );
        assert_eq!(run.installed, OLD_BINARY);
        assert_eq!(run.files, [EXE_NAME, MARKER]);
    }

    #[test]
    fn rate_limited_api_is_reported() {
        for status in [403, 429] {
            let run = update(
                &format!("limited-{}", status),
                Release {
                    api_status: status,
                    ..release("v99.0.0", OLD_BINARY)
                },
            );
            assert_eq!(run.result, Err(String::from(RATE_LIMITED)));
            assert_eq!(run.asked, 1);
            assert_eq!(run.files, [EXE_NAME]);
        }
    }

    #[cfg(windows)]
    #[test]
    fn curl_is_never_looked_up_by_bare_name() {
        let sandbox = Sandbox::new("system");
        assert_eq!(curl_in(&sandbox.dir()), None);
        let installed = sandbox.path("curl.exe");
        std::fs::write(&installed, b"").unwrap();
        assert_eq!(curl_in(&sandbox.dir()), Some(installed));
    }

    #[test]
    fn curl_skips_the_user_config_and_speaks_only_https() {
        const UNSUPPORTED_PROTOCOL: i32 = 1;
        let sandbox = Sandbox::new("curlrc");
        let [source, trace, copy, headers] =
            ["source.txt", "trace.txt", "copy.txt", "headers.txt"].map(|name| sandbox.path(name));
        std::fs::write(&source, b"x").unwrap();
        let config = format!("trace-ascii = \"{}\"\n", trace.replace('\\', "/"));
        for name in [".curlrc", "_curlrc"] {
            std::fs::write(sandbox.path(name), &config).unwrap();
        }
        let url = format!(
            "file:///{}",
            source.replace('\\', "/").trim_start_matches('/')
        );
        let output = std::process::Command::new(curl().unwrap())
            .args(curl_args("5", &headers, &copy, &url))
            .env("CURL_HOME", sandbox.dir())
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(UNSUPPORTED_PROTOCOL));
        assert!(!os::exists(&trace));
        assert!(!os::exists(&copy));
    }

    #[test]
    fn status_comes_from_the_last_response() {
        let redirected = "HTTP/1.1 302 Found\r\nLocation: x\r\n\r\nHTTP/2 200\r\nserver: y\r\n\r\n";
        assert_eq!(last_status(redirected), Some(200));
        assert_eq!(last_status("HTTP/2 403 \r\n\r\n"), Some(403));
        assert_eq!(last_status(""), None);
    }

    #[test]
    fn state_folder_that_cannot_be_created_is_an_error() {
        let sandbox = Sandbox::new("blocked");
        let blocker = sandbox.0.join("file");
        std::fs::write(&blocker, b"").unwrap();
        let dir = blocker.join("state").to_string_lossy().into_owned();
        assert_eq!(
            take_lock(&dir),
            Err(format!("dossier d'etat impossible a creer : {}\n", dir))
        );
    }

    #[test]
    fn second_lock_waits_for_the_first() {
        let sandbox = Sandbox::new("lock");
        let dir = sandbox.path("state");
        assert_eq!(take_lock(&dir), Ok(Some(os::join(&dir, &["update.lock"]))));
        assert_eq!(take_lock(&dir), Ok(None));
    }
}
