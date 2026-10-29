use super::armor::unarmor;
use super::key;

pub(crate) const TEST_KEY: &str =
    include_str!("../../../tests/fixtures/signing/throwaway-test-key.pub");
pub(crate) const MANIFEST: &[u8] =
    include_bytes!("../../../tests/fixtures/signing/throwaway-test-manifest");
pub(crate) const SIGNATURE: &[u8] =
    include_bytes!("../../../tests/fixtures/signing/throwaway-test-manifest.sig");
pub(crate) const OTHER_NAMESPACE: &[u8] =
    include_bytes!("../../../tests/fixtures/signing/throwaway-test-manifest-other-namespace.sig");
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
