use super::base64;
use alloc::vec::Vec;

const BEGIN: &[u8] = b"-----BEGIN SSH SIGNATURE-----\n";
const END: &[u8] = b"\n-----END SSH SIGNATURE-----";

pub(crate) fn unarmor(text: &[u8]) -> Option<Vec<u8>> {
    let text = text.strip_suffix(b"\n").unwrap_or(text);
    let body = text.strip_prefix(BEGIN)?.strip_suffix(END)?;
    if body.contains(&b'\r') || body.windows(2).any(|w| w == b"\n\n") {
        return None;
    }
    let joined: Vec<u8> = body.iter().copied().filter(|&c| c != b'\n').collect();
    base64::decode(&joined)
}
