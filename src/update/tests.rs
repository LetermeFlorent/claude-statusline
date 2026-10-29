use super::*;
use crate::sandbox::Sandbox;
use core::cell::RefCell;
use fetch::github;
use verify::WRONG_VERSION_EXIT;

const OLD_BINARY: &[u8] = b"old";
const WORKING_BINARY: &[u8] = b"statusline 99.0.0\n|demo";
const FAILING_BINARY: &[u8] = b"crash";
const OTHER_VERSION_BINARY: &[u8] = b"statusline 98.0.0\n|demo";
const NO_DEMO_BINARY: &[u8] = b"statusline 99.0.0\n|";
const EXE_NAME: &str = if cfg!(windows) {
    "statusline.exe"
} else {
    "statusline"
};

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
        let asset_url = format!("/download/{}/{}", self.tag, asset().unwrap());
        let (status, body) = if url.ends_with("/releases/latest") && self.api_status != HTTP_OK {
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

fn failed(message: String) -> Result<Outcome, Failure> {
    Err(Failure::from(message))
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
    let result = attempt(&sandbox.dir(), &exe, &tools);
    Run {
        result,
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
        assert_eq!(
            run.result,
            failed(String::from(messages::NO_RELEASE.text()))
        );
        assert_eq!(run.installed, OLD_BINARY);
        assert_eq!(run.files, [EXE_NAME]);
    }
}

#[test]
fn binary_failing_its_version_check_is_not_swapped_in() {
    let run = update("failing", release("v99.0.0", FAILING_BINARY));
    assert_eq!(
        run.result,
        failed(messages::UPDATE_FAILED.with(&["v99.0.0"]))
    );
    assert_eq!(run.installed, OLD_BINARY);
    assert_eq!(run.files, [EXE_NAME, MARKER]);
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
fn binary_failing_its_demo_is_not_swapped_in() {
    let run = update("no-demo", release("v99.0.0", NO_DEMO_BINARY));
    assert_eq!(
        run.result,
        failed(messages::UPDATE_FAILED.with(&["v99.0.0"]))
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
        assert_eq!(
            run.result,
            failed(String::from(messages::RATE_LIMITED.text()))
        );
        assert_eq!(run.asked, 1);
        assert_eq!(run.files, [EXE_NAME]);
    }
}
