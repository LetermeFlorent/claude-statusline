# Releasing

Every release is signed by hand with an Ed25519 key that only the owner holds. CI builds and checksums the binaries, the owner signs the manifest, and the updater and the installers refuse anything the key did not sign.

## One-time key setup

On Windows 11, with the OpenSSH client that ships with the system, run:

    ssh-keygen -t ed25519 -f %USERPROFILE%\.ssh\statusline-release -C "claude-statusline release signing"

Keep the private key off GitHub and keep an offline backup of it. The public key must be the one embedded in the program and in both installers, with the fingerprint `SHA256:eXVs/Iwpre7dVdF+oSohVqRkgj3kWU6SVRiMRgxhwCM`. Check it with `ssh-keygen -lf %USERPROFILE%\.ssh\statusline-release.pub`. The signing scripts verify every new signature against that embedded key before uploading, so a wrong key is caught early.

## Each release

1. Bump the version in `Cargo.toml` and `Cargo.lock`.
2. Add a `## X.Y.Z - date` section to `CHANGELOG.md`.
3. Merge to `main`, then tag `vX.Y.Z` on `main` and push the tag.
4. Wait for the CI run to create the draft release. It holds the six binaries, `SHA256SUMS` and `release-manifest`.
5. Run the signing script. On Windows:

        powershell -File scripts\sign-release.ps1 -Tag vX.Y.Z

    On Linux or macOS (needs `curl`, `ssh-keygen` and `python3` for the JSON):

        scripts/sign-release.sh -t vX.Y.Z

    The script asks for a GitHub token (or reads `GITHUB_TOKEN`), downloads the draft, checks every checksum against the manifest, shows the manifest, signs after you answer `y`, verifies the signature, and uploads `release-manifest.sig`. Use `-Replace` (or `-R` in the shell script) to overwrite an existing signature.
6. Read the draft, then publish it by hand as the latest release.

## Unsigned releases

A release without `release-manifest.sig`, or with a signature that does not verify, is refused by the updater and by the installers. Publishing before signing therefore leaves users on their current version.

## If the key is lost

A new key means a new program version that embeds the new public key, and installs that carry the old key cannot verify releases signed by it, so they have to be reinstalled by hand. Keep the backup in two places and test restoring it once.
