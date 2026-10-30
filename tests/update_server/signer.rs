#[path = "../../src/upgrade/sign/sha256.rs"]
mod sha256;

use ed25519_compact::sha512::Hash;
use ed25519_compact::{KeyPair, Seed};

pub const KEY_VARIABLE: &str = "STATUSLINE_TEST_RELEASE_KEY";
const SEED: [u8; 32] = [9; 32];
const NAMESPACE: &[u8] = b"statusline-release";
const KEY_TYPE: &[u8] = b"ssh-ed25519";
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const ASSETS: [&str; 4] = [
    "statusline-linux-arm64",
    "statusline-linux-x86_64",
    "statusline-windows-arm64.exe",
    "statusline-windows-x86_64.exe",
];

pub struct Signed {
    pub manifest: Vec<u8>,
    pub signature: Vec<u8>,
    pub key: String,
}

fn put(out: &mut Vec<u8>, value: &[u8]) {
    out.extend_from_slice(&u32::try_from(value.len()).unwrap().to_be_bytes());
    out.extend_from_slice(value);
}

fn base64(data: &[u8]) -> String {
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let mut word = [0u8; 3];
        word[..chunk.len()].copy_from_slice(chunk);
        let bits = u32::from_be_bytes([0, word[0], word[1], word[2]]);
        for index in 0..=chunk.len() {
            let sextet = (bits >> (18 - 6 * index)) & 63;
            out.push(char::from(ALPHABET[sextet as usize]));
        }
        out.push_str(&"=="[..3 - chunk.len()]);
    }
    out
}

pub fn sign(tag: &str, binary: &[u8]) -> Signed {
    let pair = KeyPair::from_seed(Seed::new(SEED));
    let sum: String = sha256::sha256(binary)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let mut manifest = format!("tag {tag}\n");
    for asset in ASSETS {
        manifest.push_str(&format!("{sum}  {asset}\n"));
    }
    let mut signed = b"SSHSIG".to_vec();
    let digest = Hash::hash(manifest.as_bytes());
    let fields: [&[u8]; 4] = [NAMESPACE, b"", b"sha512", &digest];
    for field in fields {
        put(&mut signed, field);
    }
    let mut inner = Vec::new();
    put(&mut inner, KEY_TYPE);
    put(&mut inner, pair.sk.sign(signed, None).as_ref());
    let mut key = Vec::new();
    put(&mut key, KEY_TYPE);
    put(&mut key, &pair.pk[..]);
    let mut blob = b"SSHSIG".to_vec();
    blob.extend_from_slice(&1u32.to_be_bytes());
    let fields: [&[u8]; 5] = [&key, NAMESPACE, b"", b"sha512", &inner];
    for field in fields {
        put(&mut blob, field);
    }
    let body = base64(&blob);
    let armor = format!("-----BEGIN SSH SIGNATURE-----\n{body}\n-----END SSH SIGNATURE-----\n");
    Signed {
        manifest: manifest.into_bytes(),
        signature: armor.into_bytes(),
        key: format!("ssh-ed25519 {} test", base64(&key)),
    }
}
