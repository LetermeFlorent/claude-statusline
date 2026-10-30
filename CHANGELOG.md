# Changelog

All notable changes to this project are listed here, newest first. Versions follow the `version` in Cargo.toml and the git tag of the same name.

## Unreleased

- Releases are signed and published by CI. The release job signs the manifest with the key held in the `release` environment, checks the signature against the key embedded in the program, then publishes the release, so updates reach users with no step by hand. When the key is missing, the job leaves an unsigned draft for the signing scripts in `scripts/`, as before. A tag whose commit is not on `main` stops the job before signing.

## 2.2.0 - 2026-10-30

Changes since v2.1.1.

- Releases are signed. Each release carries `SHA256SUMS` and a `release-manifest` that binds the tag to every file and its SHA-256 hash, signed with an Ed25519 key kept outside GitHub (OpenSSH format, namespace `statusline-release`). `--update` checks the signature, the tag, the version and the hash before it runs or installs anything, and keeps the installed program on any doubt. The install scripts check the signature with the local `ssh-keygen` when it can, and fall back to the SHA-256 check with a warning when it cannot.
- Automatic updates are on by default again, now that every update is signed and checked. `"auto_update": false` in `statusline.json` turns them off, a `statusline.json` that cannot be read keeps them off, and `--update` still works by hand.
- Updates and installers are stricter: curl runs from the system folder and ignores `~/.curlrc`, the settings file is parsed strictly before it is rewritten, and three private backups are kept before it is swapped atomically.
- The state file and the session input are bounded and cleaned: regular files up to 1 MiB, no injected lines, no endless reset loop, terminal controls stripped from the model, effort and glyphs, and numbers that overflow are refused. Several sessions saving at once no longer lose each other's quotas.
- Windows binaries build repeatably and are attested, a native Windows on Arm binary is built, and the Windows executable is no longer kept in git.
- Readings are more accurate: the disk figure is busy time on the disk that holds the path, Linux containers show the memory of their cgroup limit, and free RAM works on kernels without MemAvailable.
- The background colour follows each terminal's own settings, WSL included, and `NO_COLOR` or `TERM=dumb` gives plain text.
- The install scripts run without a clone: download the script alone and it fetches the latest release. `install.sh --uninstall` and `install.ps1 -Uninstall` remove the bar from every account, and `--install` and `--uninstall` answer each settings case on its own.
- The program no longer ships in git: `install.ps1` downloads the Windows binary of the latest release and checks it, like `install.sh`. An upgrade means running the install script again, or letting the program update itself.
- The bar speaks French on a French system and English elsewhere: units read `7j`, `2j07h`, `1h20min`, `11,3/15,3Go`, `Md` and `maj` in French, and `STATUSLINE_LANG` forces `fr` or `en`. `--language` prints the one in use. The help, the messages and the install scripts follow the same choice.
- New `spend` segment, shown in place of `5h` and `7d` when the session holds only a spend limit, as behind a Claude apps gateway (`segments.spend` hides it).
- Colours fall back to the 256-colour palette when `COLORTERM` does not announce 24-bit colour, and `COLUMNS` makes the line drop details to fit.
- The context bar is measured against the real auto-compact threshold (including `modelSettings` and `workspace.project_dir`), warns from 70 % and 90 %, and a per-project cache keeps the threshold of several sessions. The auto-compact marker is now `~` followed by a space, and the optional `cold_cache_marker` shows `cold` when the prompt cache is not warm.
- Project files on network folders are left unread (UNC paths, network drives, NFS, SMB, autofs and similar mounts), so a stalled share cannot hold the bar.
- The look changed slightly: empty squares are dimmer, text keeps the terminal's default colour, the default cpu and context blues are lighter, and rounded token counts move to the next unit at 1000.
- `--demo` takes `low`, `warn` or `hot` and shows sample disk and cpu values, and `--bench` reports the time of each stage.
- New options: `--check` validates `statusline.json`, and metric names are read without case.
- The README, the install guide and the CI step names are in English, and the sources carry no comments.
- CI runs rustfmt, clippy and the tests on all six rows (Linux, macOS and Windows, each on x86_64 and arm64), runs both installer scripts against a local release, tests the self-update against a local release, and pins the rendered line for the dark and light palettes.
- Cargo.toml now declares `rust-version`, the licence (MIT), the repository and a description. SECURITY.md and an issue template are new. The program has one build dependency, `ed25519-compact`, compiled in; the binary still needs nothing at run time.

## 2.1.1 - 2026-10-20

- Fixes the Windows build. Since Rust 1.99, LLVM replaces the loop that measures a wide string with a call to `wcslen`. The Windows binary is linked without the C runtime, so `rust-lld` could not find the symbol and CI failed before it published anything. `wcslen` now sits next to `memcpy` and `memset` in the minimal runtime.
- This is the first published release with everything listed under 2.1.0.

## 2.1.0 - 2026-10-20

Tagged, but never published: the release job failed on the `wcslen` link error described under 2.1.1, so no binary was attached to it. Use 2.1.1 or later.

- The `statusLine` command is written as `~/.claude/bin/statusline.exe` on Windows and `~/.claude/bin/statusline` elsewhere. It holds on every PC whatever the user name, and the installers replace the old absolute path.
- Automatic updates: at most every 6 hours, the bar looks for a newer version on GitHub, downloads it in the background, replaces its binary and shows `update` at the bottom meanwhile. Setting `auto_update` to `false` in `statusline.json` stops the check. (Updates became opt-in after this version, see 2.2.0.)
- The installers repair stale paths left after an account migration.
- CI refuses a tag that does not match the version in Cargo.toml.
- The documentation was reviewed line by line against the code.

## 2.0.0 - 2026-10-07

- The bar is rewritten in Rust for Windows, Linux and macOS, with the same rendering and settings as the earlier binary. The Claude Code mod is removed: the bar is a program run through `statusLine`.
- The light or dark background is detected (host terminal, `COLORFGBG`, system theme), and `--theme` prints the one in use.
- Quotas are kept per account in the state file, and the auto-compact threshold is cached per session folder.
- `--install` merges the `statusLine` block into `settings.json` and refuses a file it cannot read.
- `install.sh` installs on Linux and macOS, using the published binary when the build fails. CI builds on the three systems and publishes binaries per tag.
