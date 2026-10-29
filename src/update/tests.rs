use super::*;
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
