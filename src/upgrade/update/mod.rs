mod fetch;
mod leftovers;
mod lock;
mod origin;
mod swap;
#[cfg(test)]
mod tests;
mod verify;

use crate::messages;
use crate::os;
use crate::update_watch::{asset, newer, MARKER};
use alloc::string::String;
use fetch::{curl, latest, Curl, Fetch, HTTP_OK};
use leftovers::{clean_binaries, clean_temp};
use origin::{origin, Origin};
use swap::{staging, swap, Swap};
use verify::{check, printed, Failure, Probe, FAILED_EXIT};

pub use lock::running;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const SHOWN_AFTER_SWAP_MS: u32 = 5000;
#[cfg(windows)]
const NOT_RESTORED_EXIT: i32 = 4;

#[derive(Debug, PartialEq)]
enum Outcome {
    UpToDate,
    Updated(String),
}

struct Tools<'a> {
    origin: Origin,
    fetch: Fetch<'a>,
    probe: Probe<'a>,
}

fn install(exe: &str, tag: &str, tools: &Tools) -> Result<(), Failure> {
    let Some(name) = asset() else {
        return Err(String::from(messages::NO_ASSET.text()).into());
    };
    let id = os::pid();
    let fresh = staging(exe, id);
    let url = tools.origin.download(tag, name);
    let fetched = (tools.fetch)(&url, &fresh, 300) == Some(HTTP_OK) && os::make_executable(&fresh);
    let checked = if fetched {
        check(&fresh, tag, tools.probe)
    } else {
        Err(messages::UPDATE_FAILED.with(&[tag]).into())
    };
    if let Err(failure) = checked {
        os::remove(&fresh);
        return Err(failure);
    }
    match swap(exe, &fresh, id) {
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

fn attempt(dir: &str, exe: &str, tools: &Tools) -> Result<Outcome, Failure> {
    clean_binaries(exe, os::now());
    clean_temp(dir, os::now());
    let tag = latest(dir, &tools.origin, tools.fetch)?;
    if !newer(&tag, VERSION) {
        return Ok(Outcome::UpToDate);
    }
    os::write(&os::join(dir, &[MARKER]), tag.as_bytes());
    install(exe, &tag, tools)?;
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
        Err(failure) => os::fail(&failure.message, failure.code),
    }
}

pub fn run() -> i32 {
    let Some(dir) = os::state_dir() else {
        return os::fail(messages::NO_STATE_DIR.text(), FAILED_EXIT);
    };
    let Some(program) = curl() else {
        return os::fail(messages::CURL_MISSING.text(), FAILED_EXIT);
    };
    let _lock = match lock::take(&dir) {
        Ok(Some(lock)) => lock,
        Ok(None) => {
            os::out(messages::UPDATE_RUNNING.text());
            return 0;
        }
        Err(msg) => return os::fail(&msg, FAILED_EXIT),
    };
    let origin = origin();
    let curl = Curl {
        program,
        dir: &dir,
        protocols: origin.protocols,
    };
    let fetch = |url: &str, to: &str, secs: u32| curl.fetch(url, to, secs);
    let probe = |fresh: &str, flag: &str| printed(fresh, flag, &dir);
    let tools = Tools {
        origin,
        fetch: &fetch,
        probe: &probe,
    };
    let result = match os::exe() {
        Some(exe) => attempt(&dir, &exe, &tools),
        None => Err(String::from(messages::NO_EXE.text()).into()),
    };
    let code = report(result);
    os::remove(&os::join(&dir, &[MARKER]));
    code
}
