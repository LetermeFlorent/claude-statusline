use super::base64;
use super::wire::{put_string, Reader};
use alloc::vec::Vec;

pub(crate) const RELEASE_KEY: &str = concat!(
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAc/fOxJG/NBMFcBqV6k6pwGdOCfLsloZuAqCWBFmRtN",
    " claude-statusline release signing",
);
pub(crate) const KEY_TYPE: &[u8] = b"ssh-ed25519";

pub(crate) fn release_key() -> Option<[u8; 32]> {
    parse(RELEASE_KEY)
}

pub(crate) fn parse(line: &str) -> Option<[u8; 32]> {
    let mut fields = line.split_ascii_whitespace();
    if fields.next()?.as_bytes() != KEY_TYPE {
        return None;
    }
    let blob = base64::decode(fields.next()?.as_bytes())?;
    let mut reader = Reader::new(&blob);
    if reader.string()? != KEY_TYPE {
        return None;
    }
    let key: [u8; 32] = reader.string()?.try_into().ok()?;
    reader.finished().then_some(key)
}

pub(crate) fn blob(key: &[u8; 32]) -> Vec<u8> {
    let mut out = Vec::new();
    put_string(&mut out, KEY_TYPE);
    put_string(&mut out, key);
    out
}
