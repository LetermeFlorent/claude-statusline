mod fetch;
mod leftovers;
mod lock;
mod swap;
#[cfg(test)]
mod tests;

use crate::messages;
use crate::os;
use crate::update_watch::{asset, newer, MARKER};
use alloc::string::String;
use fetch::{curl, download_url, latest, Curl, Fetch, HTTP_OK};
use leftovers::{clean_binaries, clean_temp};
use swap::{staging, swap, Swap};

pub use lock::running;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const SHOWN_AFTER_SWAP_MS: u32 = 5000;
const FAILED_EXIT: i32 = 1;
#[cfg(windows)]
const NOT_RESTORED_EXIT: i32 = 4;

#[derive(Debug, PartialEq)]
enum Outcome {
    UpToDate,
    Updated(String),
}

#[derive(Debug, PartialEq)]
struct Failure {
    message: String,
    code: i32,
}

impl From<String> for Failure {
    fn from(message: String) -> Failure {
        Failure {
            message,
            code: FAILED_EXIT,
        }
    }
}

fn fail(msg: &str) -> i32 {
    os::err(msg);
    FAILED_EXIT
}

fn install(dir: &str, exe: &str, tag: &str, fetch: Fetch) -> Result<(), Failure> {
    let Some(name) = asset() else {
        return Err(String::from(messages::NO_ASSET.text()).into());
    };
    let id = os::process_id();
    let fresh = staging(exe, id);
    let ready = fetch(&download_url(tag, name), &fresh, 300) == Some(HTTP_OK)
        && os::make_executable(&fresh)
        && os::call(&fresh, &["--version"], dir, 30) == Some(0);
    match if ready {
        swap(exe, &fresh, id)
    } else {
        Swap::Refused
    } {
        Swap::Done => Ok(()),
        Swap::Refused => {
            if os::exists(exe) {
                os::remove(&fresh);
            }
            Err(messages::UPDATE_FAILED.with(&[tag]).into())
        }
        #[cfg(windows)]
        Swap::Lost(old) => Err(Failure {
            message: messages::NOT_RESTORED.with(&[&old, exe]),
            code: NOT_RESTORED_EXIT,
        }),
    }
}

fn attempt(dir: &str, exe: &str, fetch: Fetch) -> Result<Outcome, Failure> {
    clean_binaries(exe);
    clean_temp(dir, os::now());
    let tag = latest(dir, fetch)?;
    if !newer(&tag, VERSION) {
        return Ok(Outcome::UpToDate);
    }
    os::write(&os::join(dir, &[MARKER]), tag.as_bytes());
    install(dir, exe, &tag, fetch)?;
    Ok(Outcome::Updated(tag))
}

fn report(result: Result<Outcome, Failure>) -> i32 {
    match result {
        Ok(Outcome::UpToDate) => {
            os::out(&messages::UP_TO_DATE.with(&[VERSION]));
            0
        }
        Ok(Outcome::Updated(tag)) => {
            os::out(&messages::UPDATED.with(&[VERSION, &tag]));
            os::sleep(SHOWN_AFTER_SWAP_MS);
            0
        }
        Err(failure) => {
            os::err(&failure.message);
            failure.code
        }
    }
}

pub fn run() -> i32 {
    let Some(dir) = os::state_dir() else {
        return fail(messages::NO_STATE_DIR.text());
    };
    let Some(program) = curl() else {
        return fail(messages::CURL_MISSING.text());
    };
    let _lock = match lock::take(&dir) {
        Ok(Some(lock)) => lock,
        Ok(None) => {
            os::out(messages::UPDATE_RUNNING.text());
            return 0;
        }
        Err(msg) => return fail(&msg),
    };
    let curl = Curl { program, dir: &dir };
    let fetch = |url: &str, to: &str, secs: u32| curl.fetch(url, to, secs);
    let result = match os::exe() {
        Some(exe) => attempt(&dir, &exe, &fetch),
        None => Err(String::from(messages::NO_EXE.text()).into()),
    };
    let code = report(result);
    os::remove(&os::join(&dir, &[MARKER]));
    code
}
