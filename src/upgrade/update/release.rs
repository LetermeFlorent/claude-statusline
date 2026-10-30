use super::fetch::HTTP_OK;
use super::verify::Failure;
use super::Tools;
use crate::messages;
use crate::os::{self, Contents};
use crate::upgrade::sign::key::release_key;
use crate::upgrade::sign::manifest::asset_sum;
use crate::upgrade::sign::sha256::sha256;
use crate::upgrade::sign::sshsig::{verify, MAX_MESSAGE, MAX_SIGNATURE};
use alloc::format;
use alloc::vec::Vec;

const MANIFEST: &str = "release-manifest";
const SIGNATURE: &str = "release-manifest.sig";
const SMALL_SECS: u32 = 60;
const MAX_BINARY: usize = 64 << 20;
#[cfg(debug_assertions)]
const KEY_VARIABLE: &str = "STATUSLINE_TEST_RELEASE_KEY";

#[cfg(debug_assertions)]
fn test_key() -> Option<[u8; 32]> {
    os::env(KEY_VARIABLE).and_then(|line| crate::upgrade::sign::key::parse(&line))
}

#[cfg(not(debug_assertions))]
fn test_key() -> Option<[u8; 32]> {
    None
}

pub fn trusted_key() -> [u8; 32] {
    test_key().or_else(release_key).unwrap_or_default()
}

fn small(tools: &Tools, tag: &str, name: &str, to: &str, limit: usize) -> Option<Vec<u8>> {
    let url = tools.origin.download(tag, name);
    let fetched = (tools.fetch)(&url, to, SMALL_SECS) == Some(HTTP_OK);
    let data = if fetched { os::read(to) } else { None };
    os::remove(to);
    data.filter(|data| data.len() <= limit)
}

pub fn signed_sum(fresh: &str, tag: &str, name: &str, tools: &Tools) -> Result<[u8; 32], Failure> {
    let unsigned = || Failure::from(messages::UNSIGNED.with(&[tag]));
    let manifest = small(
        tools,
        tag,
        MANIFEST,
        &format!("{fresh}.manifest"),
        MAX_MESSAGE,
    );
    let signature = small(
        tools,
        tag,
        SIGNATURE,
        &format!("{fresh}.sig"),
        MAX_SIGNATURE,
    );
    let (Some(manifest), Some(signature)) = (manifest, signature) else {
        return Err(unsigned());
    };
    if !verify(&tools.key, &manifest, &signature) {
        return Err(unsigned());
    }
    asset_sum(&manifest, tag, name).ok_or_else(unsigned)
}

pub fn matches(fresh: &str, sum: &[u8; 32]) -> bool {
    match os::read_checked(fresh, MAX_BINARY) {
        Contents::Bytes(data) => sha256(&data) == *sum,
        _ => false,
    }
}
