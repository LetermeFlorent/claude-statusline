use alloc::format;
use alloc::string::String;

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

pub fn fingerprint<'a>(parts: impl IntoIterator<Item = &'a str>) -> String {
    let mut hash = FNV_OFFSET;
    for part in parts {
        for byte in part.bytes().chain(core::iter::once(0)) {
            hash = (hash ^ byte as u64).wrapping_mul(FNV_PRIME);
        }
    }
    format!("{:016x}", hash)
}
