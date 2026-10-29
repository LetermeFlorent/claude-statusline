use super::manifest::asset_sum;
use super::sha256::sha256;
use super::testkit::MANIFEST;

const TAG: &str = "v99.0.0";
const ASSET: &str = "statusline-linux-x86_64";
const SUM: &str = "52acac0a13c679c3257c5ca2806f543cea303904e2b2e406a9a42d30b2177176";

fn text() -> String {
    String::from_utf8(MANIFEST.to_vec()).unwrap()
}

fn sum(manifest: &str) -> Option<[u8; 32]> {
    asset_sum(manifest.as_bytes(), TAG, ASSET)
}

#[test]
fn the_signed_manifest_gives_the_sum_of_the_asset() {
    let expected = sha256(b"statusline 99.0.0\n|demo");
    assert_eq!(asset_sum(MANIFEST, TAG, ASSET), Some(expected));
    assert_eq!(asset_sum(MANIFEST, TAG, "statusline-windows-arm64.exe"), Some(expected));
}

#[test]
fn another_tag_or_a_missing_asset_is_refused() {
    assert_eq!(asset_sum(MANIFEST, "v98.0.0", ASSET), None);
    assert_eq!(asset_sum(MANIFEST, "99.0.0", ASSET), None);
    assert_eq!(asset_sum(MANIFEST, TAG, "statusline-freebsd-x86_64"), None);
    assert_eq!(asset_sum(MANIFEST, TAG, ""), None);
}

#[test]
fn an_asset_listed_twice_is_refused() {
    let twice = format!("{}{SUM}  {ASSET}\n", text());
    assert_eq!(sum(&twice), None);
}

#[test]
fn malformed_lines_are_refused() {
    let good = text();
    let line = format!("{SUM}  {ASSET}\n");
    let cases = [
        good.replace(SUM, &SUM.to_uppercase()),
        good.replace(&line, &format!("{}  {ASSET}\n", &SUM[1..])),
        good.replace(&line, &format!("{SUM} {ASSET}\n")),
        good.replace(&line, &format!("{SUM}  {ASSET} \n")),
        good.replace('\n', "\r\n"),
        String::from(good.trim_end()),
        format!("{good}\n"),
        format!("{good}notes\n"),
        good.replacen("tag ", "tag  ", 1),
        good.replacen("tag v99.0.0\n", "", 1),
        format!("tag {TAG}\n"),
    ];
    for manifest in cases {
        assert_eq!(sum(&manifest), None, "{manifest:?}");
    }
}
