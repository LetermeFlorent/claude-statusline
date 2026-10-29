use alloc::vec::Vec;

pub(crate) fn asset_sum(manifest: &[u8], tag: &str, asset: &str) -> Option<[u8; 32]> {
    let text = core::str::from_utf8(manifest).ok()?.strip_suffix('\n')?;
    let mut lines = text.split('\n');
    if lines.next()?.strip_prefix("tag ")? != tag {
        return None;
    }
    let mut names = Vec::new();
    let mut found = None;
    for line in lines {
        let (hex, name) = line.split_once("  ")?;
        let sum = digest(hex)?;
        if name.is_empty() || name.bytes().any(|c| !c.is_ascii_graphic()) || names.contains(&name) {
            return None;
        }
        names.push(name);
        if name == asset {
            found = Some(sum);
        }
    }
    found
}

fn digest(hex: &str) -> Option<[u8; 32]> {
    let bytes = hex.as_bytes();
    if bytes.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (slot, pair) in out.iter_mut().zip(bytes.chunks_exact(2)) {
        *slot = (nibble(pair[0])? << 4) | nibble(pair[1])?;
    }
    Some(out)
}

fn nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        _ => None,
    }
}
