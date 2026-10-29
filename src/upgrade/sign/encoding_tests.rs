use super::armor::unarmor;
use super::base64::decode;
use super::key::{blob, parse, release_key};
use super::sha256::sha256;
use super::testkit::{encode, test_key, SIGNATURE, TEST_KEY};

#[test]
fn base64_decodes_padded_and_full_groups() {
    assert_eq!(decode(b"QQ=="), Some(b"A".to_vec()));
    assert_eq!(decode(b"QUI="), Some(b"AB".to_vec()));
    assert_eq!(decode(b"QUJD"), Some(b"ABC".to_vec()));
    let data: Vec<u8> = (0..=255).collect();
    for len in 1..40 {
        let text = encode(&data[..len], true);
        assert_eq!(decode(text.as_bytes()), Some(data[..len].to_vec()));
    }
}

#[test]
fn base64_refuses_anything_not_canonical() {
    let bad: [&[u8]; 10] = [
        b"",
        b"QQ",
        b"QQ=",
        b"QR==",
        b"QUJ=",
        b"Q===",
        b"====",
        b"QQ==QUJD",
        b"QU*D",
        b"QUJD\n",
    ];
    for text in bad {
        assert_eq!(decode(text), None, "{text:?}");
    }
}

#[test]
fn armor_refuses_carriage_returns_and_missing_markers() {
    let text = String::from_utf8(SIGNATURE.to_vec()).unwrap();
    assert!(unarmor(SIGNATURE).is_some());
    assert!(unarmor(text.replace('\n', "\r\n").as_bytes()).is_none());
    assert!(unarmor(text.replace("BEGIN SSH", "BEGIN PGP").as_bytes()).is_none());
    assert!(unarmor(text.replace("-----END SSH SIGNATURE-----", "").as_bytes()).is_none());
}

#[test]
fn the_embedded_release_key_has_the_published_fingerprint() {
    let fingerprint = encode(&sha256(&blob(&release_key().unwrap())), false);
    assert_eq!(fingerprint, "eXVs/Iwpre7dVdF+oSohVqRkgj3kWU6SVRiMRgxhwCM");
}

#[test]
fn key_lines_parse_only_as_ed25519() {
    assert_eq!(parse(TEST_KEY), Some(test_key()));
    assert_eq!(parse(&TEST_KEY.replace("ssh-ed25519 ", "ssh-rsa ")), None);
    assert_eq!(parse("ssh-ed25519 QUJD"), None);
    assert_eq!(parse(""), None);
}
