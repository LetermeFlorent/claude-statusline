mod fetch;
mod swap;
#[cfg(test)]
mod tests;

use crate::messages;
use crate::os;
use crate::state::recent;
use crate::update_watch::{asset, newer, MARKER, STALE_AFTER};
use alloc::string::String;
use fetch::{curl, download_url, latest, Curl, Fetch, HTTP_OK};
use swap::{clean, staging, swap};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const SHOWN_AFTER_SWAP_MS: u32 = 5000;

#[derive(Debug, PartialEq)]
enum Outcome {
    UpToDate,
    Updated(String),
}

fn fail(msg: &str) -> i32 {
    os::err(msg);
    1
}

fn install(dir: &str, exe: &str, tag: &str, fetch: Fetch) -> Result<(), String> {
    let Some(name) = asset() else {
        return Err(String::from(messages::NO_ASSET.text()));
    };
    let fresh = staging(exe);
    let ready = fetch(&download_url(tag, name), &fresh, 300) == Some(HTTP_OK)
        && os::make_executable(&fresh)
        && os::call(&fresh, &["--version"], dir, 30) == Some(0);
    if !ready || !swap(exe, &fresh) {
        os::remove(&fresh);
        return Err(messages::UPDATE_FAILED.with(&[tag]));
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
            os::out(&messages::UP_TO_DATE.with(&[VERSION]));
            0
        }
        Ok(Outcome::Updated(tag)) => {
            os::out(&messages::UPDATED.with(&[VERSION, &tag]));
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
    let stale = os::stamp(lock).is_none_or(|(_, at)| !recent(at as f64, os::now(), STALE_AFTER));
    stale && os::remove(lock) && os::create_private(lock, &[])
}

fn take_lock(dir: &str) -> Result<Option<String>, String> {
    if !os::mkdirs(dir) {
        return Err(messages::STATE_DIR_FAILED.with(&[dir]));
    }
    let lock = os::join(dir, &["update.lock"]);
    Ok(acquire(&lock).then_some(lock))
}

pub fn run() -> i32 {
    let Some(dir) = os::state_dir() else {
        return fail(messages::NO_STATE_DIR.text());
    };
    let Some(program) = curl() else {
        return fail(messages::CURL_MISSING.text());
    };
    let lock = match take_lock(&dir) {
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
        None => Err(String::from(messages::NO_EXE.text())),
    };
    let code = report(result);
    os::remove(&os::join(&dir, &[MARKER]));
    os::remove(&lock);
    code
}
