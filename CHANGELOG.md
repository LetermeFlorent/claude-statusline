# Changelog

All notable changes to this project are listed here, newest first. Versions follow the `version` in Cargo.toml and the git tag of the same name.

## Unreleased

Changes since v2.1.1, on the cleanup branches.

- Automatic updates are now off by default and stay opt-in until releases are signed. `--update` still works when asked for.
- Updates and installers are stricter: downloads are checked, curl runs from the system folder and ignores `~/.curlrc`, the settings file is parsed strictly before it is rewritten, and three private backups are kept before it is swapped atomically.
- The state file and the session input are bounded and cleaned: regular files up to 1 MiB, no injected lines, no endless reset loop, terminal controls stripped from the model, effort and glyphs, and numbers that overflow are refused.
- Windows binaries build repeatably and are attested, a native Windows on Arm binary is built, and the Windows executable is no longer kept in git.
- Readings are more accurate: the disk figure is busy time on the disk that holds the path, Linux containers show the memory of their cgroup limit, and free RAM works on kernels without MemAvailable.
- The background colour follows each terminal's own settings, WSL included, and `NO_COLOR` or `TERM=dumb` gives plain text.
- New options: `--check` validates `statusline.json`, and metric names are read without case.
- The help, the messages and the install scripts speak French or English.
- The README, the install guide and the CI step names are in English, and the sources carry no comments.
- CI runs rustfmt, clippy and the tests on all five rows, runs both installer scripts, tests the self-update against a local release, and pins the rendered line for the dark and light palettes.
- Cargo.toml now declares `rust-version`, the licence (MIT), the repository and a description. SECURITY.md and an issue template are new.

## 2.1.1 - 2026-10-20

- Fixes the Windows build. Since Rust 1.99, LLVM replaces the loop that measures a wide string with a call to `wcslen`. The Windows binary is linked without the C runtime, so `rust-lld` could not find the symbol and CI failed before it published anything. `wcslen` now sits next to `memcpy` and `memset` in the minimal runtime.
- This is the first published release with everything listed under 2.1.0.

## 2.1.0 - 2026-10-20

Tagged, but never published: the release job failed on the `wcslen` link error described under 2.1.1, so no binary was attached to it. Use 2.1.1 or later.

- The `statusLine` command is written as `~/.claude/bin/statusline.exe` on Windows and `~/.claude/bin/statusline` elsewhere. It holds on every PC whatever the user name, and the installers replace the old absolute path.
- Automatic updates: at most every 6 hours, the bar looks for a newer version on GitHub, downloads it in the background, replaces its binary and shows `update` at the bottom meanwhile. Setting `auto_update` to `false` in `statusline.json` stops the check. (Updates became opt-in after this version, see Unreleased.)
- The installers repair stale paths left after an account migration.
- CI refuses a tag that does not match the version in Cargo.toml.
- The documentation was reviewed line by line against the code.

## 2.0.0 - 2026-10-07

- The bar is rewritten in Rust for Windows, Linux and macOS, with the same rendering and settings as the earlier binary. The Claude Code mod is removed: the bar is a program run through `statusLine`.
- The light or dark background is detected (host terminal, `COLORFGBG`, system theme), and `--theme` prints the one in use.
- Quotas are kept per account in the state file, and the auto-compact threshold is cached per session folder.
- `--install` merges the `statusLine` block into `settings.json` and refuses a file it cannot read.
- `install.sh` installs on Linux and macOS, using the published binary when the build fails. CI builds on the three systems and publishes binaries per tag.
