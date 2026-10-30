use super::*;
use crate::sandbox::Sandbox;
use core::cell::RefCell;
use origin::github;
use verify::WRONG_VERSION_EXIT;

const OLD_BINARY: &[u8] = b"old";
const WORKING_BINARY: &[u8] = b"statusline 99.0.0\n|demo";
const FAILING_BINARY: &[u8] = b"crash";
const OTHER_VERSION_BINARY: &[u8] = b"statusline 98.0.0\n|demo";
const NO_DEMO_BINARY: &[u8] = b"statusline 99.0.0\n|";
const RATE_LIMITED_BODY: &[u8] = b"{\"message\": \"API rate limit exceeded\"}";
#[cfg(windows)]
const EXE_NAME: &str = "statusline.exe";
#[cfg(not(windows))]
const EXE_NAME: &str = "statusline";

fn run_fake(exe: &str, flag: &str) -> Option<String> {
    let body = std::fs::read_to_string(exe).ok()?;
    let (version, demo) = body.split_once('|')?;
    match flag {
        "--version" => Some(String::from(version)),
        "--demo" => (!demo.is_empty()).then(String::new),
        _ => None,
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
        let release = format!("{{\"tag_name\": \"{}\"}}", self.tag).into_bytes();
        let asset_url = format!("/download/{}/{}", self.tag, asset().unwrap());
        let (status, body) = match (url.ends_with("/releases/latest"), self.api_status) {
            (true, HTTP_OK) => (HTTP_OK, &release[..]),
            (true, limited) => (limited, RATE_LIMITED_BODY),
            _ if url.ends_with(&asset_url) => (HTTP_OK, self.binary),
            _ => return None,
        };
        std::fs::write(to, body).ok().map(|_| status)
    }
}

fn failed(message: &str) -> Result<Outcome, Failure> {
    Err(Failure::from(String::from(message)))
}

struct Run {
    result: Result<Outcome, Failure>,
    installed: Vec<u8>,
    files: Vec<String>,
    asked: usize,
}

fn update(name: &str, release: Release) -> Run {
    let sandbox = Sandbox::new(name);
    let exe = sandbox.path(EXE_NAME);
    std::fs::write(&exe, OLD_BINARY).unwrap();
    let tools = Tools {
        origin: github(),
        fetch: &|u, t, s| release.serve(u, t, s),
        probe: &run_fake,
    };
    Run {
        result: attempt(&sandbox.dir(), &exe, &tools),
        installed: std::fs::read(&exe).unwrap(),
        files: sandbox.files(),
        asked: release.asked.into_inner().len(),
    }
}

#[test]
fn newer_tag_is_installed() {
    let run = update("newer", release("v99.0.0", WORKING_BINARY));
    assert_eq!(run.result, Ok(Outcome::Updated(String::from("v99.0.0"))));
    assert_eq!(run.installed, WORKING_BINARY);
    assert!(!run.files.iter().any(|f| f.contains(".new")));
}

#[test]
fn same_tag_is_already_up_to_date() {
    let run = update("same", release(&format!("v{}", VERSION), WORKING_BINARY));
    assert_eq!(run.result, Ok(Outcome::UpToDate));
    assert_eq!(run.installed, OLD_BINARY);
    assert_eq!(run.asked, 1);
    assert_eq!(run.files, [EXE_NAME]);
}

#[test]
fn tag_outside_x_y_z_is_refused() {
    for (i, tag) in ["v99.0", "v99.0.0-rc1", "latest"].iter().enumerate() {
        let run = update(&format!("refused-{}", i), release(tag, WORKING_BINARY));
        assert_eq!(run.result, failed(messages::NO_RELEASE.text()));
        assert_eq!(run.installed, OLD_BINARY);
        assert_eq!(run.files, [EXE_NAME]);
    }
}

#[test]
fn binary_reporting_another_version_is_refused_with_its_own_code() {
    let run = update("other-version", release("v99.0.0", OTHER_VERSION_BINARY));
    let message = messages::WRONG_VERSION.with(&["statusline 98.0.0", "v99.0.0"]);
    let code = WRONG_VERSION_EXIT;
    assert_eq!(run.result, Err(Failure { message, code }));
    assert_eq!(run.installed, OLD_BINARY);
    assert_eq!(run.files, [EXE_NAME, MARKER]);
}

#[test]
fn binaries_failing_their_version_or_demo_are_not_swapped_in() {
    for (name, binary) in [("failing", FAILING_BINARY), ("no-demo", NO_DEMO_BINARY)] {
        let run = update(name, release("v99.0.0", binary));
        let refused = messages::UPDATE_FAILED.with(&["v99.0.0"]);
        assert_eq!(run.result, failed(&refused));
        assert_eq!(run.installed, OLD_BINARY);
        assert_eq!(run.files, [EXE_NAME, MARKER]);
    }
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
        assert_eq!(run.result, failed(messages::RATE_LIMITED.text()));
        assert_eq!(run.asked, 1);
        assert_eq!(run.files, [EXE_NAME]);
    }
}
