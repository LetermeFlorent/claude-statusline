use super::fake::{failed, release, update, Release, EXE_NAME, OLD_BINARY, WORKING_BINARY};
use super::*;
use verify::WRONG_VERSION_EXIT;

const FAILING_BINARY: &[u8] = b"crash";
const OTHER_VERSION_BINARY: &[u8] = b"statusline 98.0.0\n|demo";
const NO_DEMO_BINARY: &[u8] = b"statusline 99.0.0\n|";

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
