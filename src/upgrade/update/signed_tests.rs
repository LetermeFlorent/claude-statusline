use super::fake::{
    failed, manifest, release, update, Release, EXE_NAME, OLD_BINARY, WORKING_BINARY,
};
use super::*;
use crate::upgrade::sign::sshsig::{MAX_MESSAGE, MAX_SIGNATURE, NAMESPACE};
use crate::upgrade::sign::testkit::{sign, test_key, MANIFEST, OTHER_NAMESPACE, SIGNATURE};

const TAG: &str = "v99.0.0";

fn refused(name: &str, release: Release, message: &str) {
    let run = update(name, release);
    assert_eq!(run.result, failed(message));
    assert_eq!(run.installed, OLD_BINARY);
    assert_eq!(run.probes, 0);
    assert_eq!(run.files, [EXE_NAME, MARKER]);
}

fn unsigned() -> String {
    messages::UNSIGNED.with(&[TAG])
}

fn not_downloaded() -> String {
    messages::NO_CHECK_FILES.with(&[TAG])
}

fn fixture(signature: &[u8]) -> Release<'static> {
    Release {
        manifest: MANIFEST.to_vec(),
        signature: signature.to_vec(),
        key: test_key(),
        ..release(TAG, WORKING_BINARY)
    }
}

fn signed(manifest: Vec<u8>) -> Release<'static> {
    Release {
        signature: sign(NAMESPACE, &manifest),
        manifest,
        ..release(TAG, WORKING_BINARY)
    }
}

#[test]
fn a_release_signed_with_ssh_keygen_is_installed() {
    let run = update("fixture", fixture(SIGNATURE));
    assert_eq!(run.result, Ok(Outcome::Updated(String::from(TAG))));
    assert_eq!(run.installed, WORKING_BINARY);
    assert_eq!(run.probes, 2);
}

#[test]
fn an_update_whose_signature_does_not_match_is_refused_and_the_old_binary_stays() {
    let other = manifest(TAG, b"statusline 99.0.0\n|other");
    let release = Release {
        signature: sign(NAMESPACE, &other),
        ..release(TAG, WORKING_BINARY)
    };
    refused("mismatch", release, &unsigned());
}

#[test]
fn a_missing_signature_or_manifest_is_refused() {
    let no_signature = Release {
        signature: Vec::new(),
        ..release(TAG, WORKING_BINARY)
    };
    refused("no-signature", no_signature, &not_downloaded());
    let no_manifest = Release {
        manifest: Vec::new(),
        ..release(TAG, WORKING_BINARY)
    };
    refused("no-manifest", no_manifest, &not_downloaded());
}

#[test]
fn check_files_over_their_size_limit_are_refused_as_not_downloaded() {
    let mut manifest = release(TAG, WORKING_BINARY).manifest;
    manifest.resize(MAX_MESSAGE + 1, b'\n');
    refused("big-manifest", signed(manifest), &not_downloaded());
    let mut signature = release(TAG, WORKING_BINARY).signature;
    signature.resize(MAX_SIGNATURE + 1, b'\n');
    let release = Release {
        signature,
        ..release(TAG, WORKING_BINARY)
    };
    refused("big-signature", release, &not_downloaded());
}

#[test]
fn a_signature_in_another_namespace_is_refused() {
    refused("namespace", fixture(OTHER_NAMESPACE), &unsigned());
    let base = release(TAG, WORKING_BINARY);
    let signature = sign(b"other-namespace", &base.manifest);
    refused("namespace-own", Release { signature, ..base }, &unsigned());
}

#[test]
fn a_manifest_of_another_release_is_refused() {
    refused(
        "other-tag",
        signed(manifest("v98.0.0", WORKING_BINARY)),
        &unsigned(),
    );
}

#[test]
fn an_asset_missing_from_the_manifest_is_refused() {
    let listed = format!("tag {TAG}\n{}  statusline-plan9-x86_64\n", "0".repeat(64));
    refused("no-asset", signed(listed.into_bytes()), &unsigned());
}

#[test]
fn a_download_that_differs_from_its_signed_sum_is_refused() {
    let release = Release {
        binary: b"statusline 99.0.0\n|tampered",
        ..release(TAG, WORKING_BINARY)
    };
    refused("sum", release, &messages::SUM_MISMATCH.with(&[TAG]));
}

#[test]
fn a_lower_version_is_never_installed() {
    let run = update("lower", release("v0.0.1", WORKING_BINARY));
    assert_eq!(run.result, Ok(Outcome::UpToDate));
    assert_eq!(run.installed, OLD_BINARY);
    assert_eq!(run.probes, 0);
    assert_eq!(run.asked, 1);
}
