use crate::json::{self, Syntax};
use crate::messages;
use crate::os;
use crate::update_watch::parts;
use alloc::format;
use alloc::string::String;

const REPO: &str = "LetermeFlorent/claude-statusline";
pub const HTTP_OK: u16 = 200;
const RATE_LIMIT_STATUSES: [u16; 2] = [403, 429];
const HTTPS_ONLY: &str = "=https";

pub type Fetch<'a> = &'a dyn Fn(&str, &str, u32) -> Option<u16>;

#[cfg(windows)]
fn curl_in(system: &str) -> Option<String> {
    let path = os::join(system, &["curl.exe"]);
    os::exists(&path).then_some(path)
}

#[cfg(windows)]
pub fn curl() -> Option<String> {
    curl_in(&os::system_dir()?)
}

#[cfg(not(windows))]
pub fn curl() -> Option<String> {
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

pub struct Curl<'a> {
    pub program: String,
    pub dir: &'a str,
}

impl Curl<'_> {
    pub fn fetch(&self, url: &str, to: &str, secs: u32) -> Option<u16> {
        let headers = format!("{}.headers", to);
        let limit = format!("{}", secs);
        let args = curl_args(&limit, &headers, to, url);
        let finished = os::call(&self.program, &args, self.dir, secs + 15) == Some(0);
        let status = os::read_text(&headers).and_then(|h| last_status(&h));
        os::remove(&headers);
        status.filter(|_| finished)
    }
}

pub fn latest(dir: &str, fetch: Fetch) -> Result<String, String> {
    let file = os::join(dir, &["release.json"]);
    let url = format!("https://api.github.com/repos/{}/releases/latest", REPO);
    let status = fetch(&url, &file, 30);
    let text = os::read_text(&file);
    os::remove(&file);
    if status.is_some_and(|s| RATE_LIMIT_STATUSES.contains(&s)) {
        return Err(String::from(messages::RATE_LIMITED.text()));
    }
    let tag = text
        .filter(|_| status == Some(HTTP_OK))
        .and_then(|t| json::parse(&t, Syntax::Tolerant))
        .and_then(|release| release.get("tag_name")?.str().map(String::from))
        .filter(|tag| parts(tag).is_some());
    tag.ok_or_else(|| String::from(messages::NO_RELEASE.text()))
}

pub fn download_url(tag: &str, name: &str) -> String {
    format!(
        "https://github.com/{}/releases/download/{}/{}",
        REPO, tag, name
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::Sandbox;

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
}
