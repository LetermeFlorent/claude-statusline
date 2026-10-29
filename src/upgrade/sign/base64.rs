use alloc::vec::Vec;

fn value(c: u8) -> Option<u32> {
    match c {
        b'A'..=b'Z' => Some(u32::from(c - b'A')),
        b'a'..=b'z' => Some(u32::from(c - b'a') + 26),
        b'0'..=b'9' => Some(u32::from(c - b'0') + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

pub(crate) fn decode(text: &[u8]) -> Option<Vec<u8>> {
    if text.is_empty() || !text.len().is_multiple_of(4) {
        return None;
    }
    let last = text.len() / 4 - 1;
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    for (index, quad) in text.chunks_exact(4).enumerate() {
        let pad = quad.iter().rev().take_while(|&&c| c == b'=').count();
        if pad > 2 || (pad > 0 && index != last) {
            return None;
        }
        let mut word = 0u32;
        for &c in &quad[..4 - pad] {
            word = (word << 6) | value(c)?;
        }
        word <<= 6 * pad as u32;
        let bytes = word.to_be_bytes();
        let keep = 1 + 3 - pad;
        if bytes[keep..].iter().any(|&b| b != 0) {
            return None;
        }
        out.extend_from_slice(&bytes[1..keep]);
    }
    Some(out)
}
