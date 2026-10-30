# Releasing

Every release is signed with an Ed25519 key, and the program's own updater refuses anything the key did not sign. CI builds and checksums the binaries, signs the manifest with the key held in the `release` environment, checks the signature against the public key embedded in the program, and publishes the release. The installers check the signature when they find a usable `ssh-keygen`, and otherwise warn and check only the SHA-256 hash from the manifest.

## Where the key lives

The private key is the environment secret `RELEASE_SIGNING_KEY` of the `release` environment on GitHub. A deployment policy limits that environment to tags matching `v*`, so a branch or a pull request cannot reach the secret. The secret holds the whole OpenSSH private key file, without a passphrase. The owner keeps a backup of the same key at `%USERPROFILE%\.ssh\statusline-release`.

That policy filters on the tag name only. It does not check who pushed the tag or which commit it points to, so anyone who can push a `v*` tag can now publish a signed release that every install updates to. A token with only Contents: write is enough for that. Two barriers close it:

- A repository ruleset on `refs/tags/v*` with Restrict creations, Restrict updates, Restrict deletions and Block force pushes, with only the repository admin role in the bypass list. It is the one barrier that also holds when the workflow file at the tag has been changed, because a changed `ci.yml` can read the secret itself.
- The release job stops before signing when the tagged commit is not on `main`. That blocks a tag pushed on a side branch, as long as `main` is protected against direct pushes too.

The public key is embedded in the program, in both installers and in the release notes, with the fingerprint `SHA256:eXVs/Iwpre7dVdF+oSohVqRkgj3kWU6SVRiMRgxhwCM`. Check the backup with `ssh-keygen -lf %USERPROFILE%\.ssh\statusline-release.pub`. Before it signs, the release job reads the key from `src/upgrade/sign/key.rs` and stops when `install.sh`, `install.ps1` or `.github/release-notes.sh` holds another one.

The key was made once, on Windows 11 with the OpenSSH client that ships with the system:

    ssh-keygen -t ed25519 -f %USERPROFILE%\.ssh\statusline-release -C "claude-statusline release signing"

## Each release

1. Bump the version in `Cargo.toml` and `Cargo.lock`.
2. Add a `## X.Y.Z - date` section to `CHANGELOG.md`.
3. Merge to `main`, then tag `vX.Y.Z` on `main` and push the tag.

CI does the rest. The release job builds the six binaries, writes `SHA256SUMS` and `release-manifest`, signs the manifest with `.github/sign-manifest.sh`, verifies that signature against the embedded public key, and only then creates the release with `release-manifest.sig` attached. A tag such as `v1.2.3` is published as the latest release, and the updater installs it at its next check. A tag with a `-` in it, such as `v1.2.3-rc.1`, is published as a pre-release and does not become the latest one.

A signature that fails, or a key that does not match the embedded one, stops the job before anything is published.

## Signing by hand

The scripts in `scripts/` are the fallback. When `RELEASE_SIGNING_KEY` is empty or missing, the job warns, skips the signature and creates a draft release as before. Sign that draft with the backup key, then read it and publish it by hand as the latest release. On Windows:

    powershell -File scripts\sign-release.ps1 -Tag vX.Y.Z

On Linux or macOS (needs `curl`, `ssh-keygen` and `python3` for the JSON):

    scripts/sign-release.sh -t vX.Y.Z

The script asks for a GitHub token (or reads `GITHUB_TOKEN`), downloads the draft, checks every checksum against the manifest and `SHA256SUMS` against the manifest lines, shows the manifest, signs after you answer `y`, verifies the signature, and uploads `release-manifest.sig`. Use `-Replace` (or `-R` in the shell script) to overwrite an existing signature on a draft. The scripts only take `vX.Y.Z` tags and only touch drafts.

## Unsigned releases

A release without `release-manifest.sig`, or with a signature that does not verify, is refused by the updater, and by the installers that have a usable `ssh-keygen`. Installers without one warn and go on with the SHA-256 hash from the manifest only. Publishing a draft before signing it therefore leaves updating users on their current version.

## If the key is lost or leaked

When the secret is lost, put the backup back into `RELEASE_SIGNING_KEY`. When the backup is gone too, the key has to be replaced, with the same cost as a leak. Keep the backup in two places and test restoring it once.

A leaked key, from the PC backup or from GitHub, lets whoever holds it sign a manifest that every install accepts, as long as they can also get it published as a release of this repository. The fix is to rotate it. Make a new key, replace the secret and the backup, embed the new public half in the program, both installers, the signing scripts and the release notes, and ship that as a new program version. Installs that carry the old key cannot verify a release signed by the new one, so they stay on their version, still trusting the old key, until they are reinstalled by hand with the new install script. Say so in the release notes and in the README.
