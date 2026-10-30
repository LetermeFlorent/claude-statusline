use super::armor::unarmor;
use super::key::{self, KEY_TYPE};
use super::sha512::sha512;
use super::wire::put_string;
use ed25519_compact::{KeyPair, Seed};

pub(crate) const TEST_KEY: &str =
    include_str!("../../../tests/fixtures/signing/throwaway-test-key.pub");
pub(crate) const MANIFEST: &[u8] =
    include_bytes!("../../../tests/fixtures/signing/throwaway-test-manifest");
pub(crate) const SIGNATURE: &[u8] =
    include_bytes!("../../../tests/fixtures/signing/throwaway-test-manifest.sig");
pub(crate) const OTHER_NAMESPACE: &[u8] =
    include_bytes!("../../../tests/fixtures/signing/throwaway-test-manifest-other-namespace.sig");
const SIGNER_SEED: [u8; 32] = [7; 32];
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub(crate) fn test_key() -> [u8; 32] {
    key::parse(TEST_KEY).unwrap()
}

pub(crate) fn encode(data: &[u8], pad: bool) -> String {
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let mut word = [0u8; 3];
        word[..chunk.len()].copy_from_slice(chunk);
        let bits = u32::from_be_bytes([0, word[0], word[1], word[2]]);
        for index in 0..=chunk.len() {
            let sextet = (bits >> (18 - 6 * index)) & 63;
            out.push(char::from(ALPHABET[sextet as usize]));
        }
        if pad {
            out.push_str(&"=="[..3 - chunk.len()]);
        }
    }
    out
}

pub(crate) fn armor(blob: &[u8]) -> Vec<u8> {
    let body = encode(blob, true);
    format!("-----BEGIN SSH SIGNATURE-----\n{body}\n-----END SSH SIGNATURE-----\n").into_bytes()
}

pub(crate) fn fixture_blob() -> Vec<u8> {
    unarmor(SIGNATURE).unwrap()
}

pub(crate) fn signer_key() -> [u8; 32] {
    *KeyPair::from_seed(Seed::new(SIGNER_SEED)).pk
}

pub(crate) fn sign(namespace: &[u8], message: &[u8]) -> Vec<u8> {
    let pair = KeyPair::from_seed(Seed::new(SIGNER_SEED));
    let digest = sha512(message);
    let mut signed = b"SSHSIG".to_vec();
    let fields: [&[u8]; 4] = [namespace, b"", b"sha512", &digest];
    for field in fields {
        put_string(&mut signed, field);
    }
    let mut inner = Vec::new();
    put_string(&mut inner, KEY_TYPE);
    put_string(&mut inner, pair.sk.sign(signed, None).as_ref());
    let mut blob = b"SSHSIG".to_vec();
    blob.extend_from_slice(&1u32.to_be_bytes());
    let fields: [&[u8]; 5] = [&key::blob(&pair.pk), namespace, b"", b"sha512", &inner];
    for field in fields {
        put_string(&mut blob, field);
    }
    armor(&blob)
}
