use super::key::release_key;
use super::sshsig::{verify, MAX_MESSAGE, MAX_SIGNATURE};
use super::testkit::{armor, fixture_blob, test_key, MANIFEST, OTHER_NAMESPACE, SIGNATURE};
use ed25519_compact::{PublicKey, Signature};

const RFC_KEY_1: &str = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
        .collect()
}

#[test]
fn ed25519_accepts_the_rfc_8032_vectors() {
    let vectors = [
        (
            RFC_KEY_1,
            "",
            "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b",
        ),
        (
            "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
            "72",
            "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00",
        ),
        (
            "fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025",
            "af82",
            "6291d657deec24024827e69c3abe01a30ce548a284743a445e3680d7db5ac3ac18ff9b538d16f290ae67f760984dc6594a7c15e9716ed28dc027beceea1ec40a",
        ),
    ];
    for (key, message, signature) in vectors {
        let key = PublicKey::from_slice(&bytes(key)).unwrap();
        let signature = Signature::from_slice(&bytes(signature)).unwrap();
        assert!(key.verify(bytes(message), &signature).is_ok());
        let mut tampered = bytes(message);
        tampered.push(0);
        assert!(key.verify(tampered, &signature).is_err());
    }
}

#[test]
fn the_ssh_keygen_fixture_verifies() {
    assert!(verify(&test_key(), MANIFEST, SIGNATURE));
    assert!(verify(&test_key(), MANIFEST, &armor(&fixture_blob())));
}

#[test]
fn another_namespace_or_another_key_is_refused() {
    let other: [u8; 32] = bytes(RFC_KEY_1).try_into().unwrap();
    assert!(!verify(&test_key(), MANIFEST, OTHER_NAMESPACE));
    assert!(!verify(&other, MANIFEST, SIGNATURE));
    assert!(!verify(&release_key().unwrap(), MANIFEST, SIGNATURE));
}

#[test]
fn a_flipped_byte_in_the_message_is_refused() {
    for at in [0, 5, MANIFEST.len() - 1] {
        let mut message = MANIFEST.to_vec();
        message[at] ^= 1;
        assert!(!verify(&test_key(), &message, SIGNATURE));
    }
}

#[test]
fn a_truncated_or_damaged_signature_is_refused() {
    let text = String::from_utf8(SIGNATURE.to_vec()).unwrap();
    let end = text.find("\n-----END").unwrap();
    for cut in [1, 2, 4, 8] {
        let short = format!("{}{}", &text[..end - cut], &text[end..]);
        assert!(!verify(&test_key(), MANIFEST, short.as_bytes()));
    }
    assert!(!verify(&test_key(), MANIFEST, &SIGNATURE[..SIGNATURE.len() / 2]));
    let damaged = text.replacen("U1NI", "U1N*", 1);
    assert!(!verify(&test_key(), MANIFEST, damaged.as_bytes()));
}

#[test]
fn extra_fields_and_other_hashes_are_refused() {
    let mut extra = fixture_blob();
    extra.extend_from_slice(&[0, 0, 0, 0]);
    assert!(!verify(&test_key(), MANIFEST, &armor(&extra)));
    let mut hash = fixture_blob();
    let at = hash.windows(6).position(|w| w == b"sha512").unwrap();
    hash[at..at + 6].copy_from_slice(b"sha256");
    assert!(!verify(&test_key(), MANIFEST, &armor(&hash)));
    let mut version = fixture_blob();
    version[9] = 2;
    assert!(!verify(&test_key(), MANIFEST, &armor(&version)));
}

#[test]
fn oversize_input_is_refused() {
    let mut message = MANIFEST.to_vec();
    message.resize(MAX_MESSAGE + 1, b'\n');
    assert!(!verify(&test_key(), &message, SIGNATURE));
    let mut signature = SIGNATURE.to_vec();
    signature.resize(MAX_SIGNATURE + 1, b'\n');
    assert!(!verify(&test_key(), MANIFEST, &signature));
}
