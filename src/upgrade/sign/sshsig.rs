use super::armor::unarmor;
use super::key::{blob, KEY_TYPE};
use super::sha512::sha512;
use super::wire::{put_string, Reader};
use alloc::vec::Vec;
use ed25519_compact::{PublicKey, Signature};

pub(crate) const NAMESPACE: &[u8] = b"statusline-release";
pub(crate) const MAX_SIGNATURE: usize = 8 * 1024;
pub(crate) const MAX_MESSAGE: usize = 64 * 1024;
const MAGIC: &[u8] = b"SSHSIG";
const VERSION: u32 = 1;
const HASH: &[u8] = b"sha512";

pub(crate) fn verify(trusted: &[u8; 32], message: &[u8], armored: &[u8]) -> bool {
    message.len() <= MAX_MESSAGE
        && armored.len() <= MAX_SIGNATURE
        && check(trusted, message, armored).is_some()
}

fn check(trusted: &[u8; 32], message: &[u8], armored: &[u8]) -> Option<()> {
    let decoded = unarmor(armored)?;
    let mut outer = Reader::new(&decoded);
    (outer.take(MAGIC.len())? == MAGIC && outer.u32()? == VERSION).then_some(())?;
    field(&mut outer, &blob(trusted))?;
    field(&mut outer, NAMESPACE)?;
    field(&mut outer, b"")?;
    field(&mut outer, HASH)?;
    let mut inner = Reader::new(outer.string()?);
    field(&mut inner, KEY_TYPE)?;
    let signature = Signature::from_slice(inner.string()?).ok()?;
    (outer.finished() && inner.finished()).then_some(())?;
    PublicKey::new(*trusted).verify(signed_data(message), &signature).ok()
}

fn field(reader: &mut Reader, expected: &[u8]) -> Option<()> {
    (reader.string()? == expected).then_some(())
}

fn signed_data(message: &[u8]) -> Vec<u8> {
    let mut out = Vec::from(MAGIC);
    put_string(&mut out, NAMESPACE);
    put_string(&mut out, b"");
    put_string(&mut out, HASH);
    put_string(&mut out, &sha512(message));
    out
}
