# Installing claude-statusline

This guide installs the status line in Claude Code, then shows how to check it, configure it, update it and remove it, on Windows, Linux and macOS. The [README](README.md) describes the display and every setting in detail.

## Before you start

You need Claude Code installed. The rest depends on the system:

- Windows: nothing else. The script downloads the released binary for Windows 10 or later, x86_64 or arm64 depending on the processor. Rust is only needed to build it yourself with `-Build`.
- Linux and macOS: `install.sh` compiles the sources when Rust is installed (https://rustup.rs). Without Rust, it downloads the released binary with `curl`, for Linux x86_64 or arm64 and for macOS on Apple silicon or Intel.

The bar sends 24-bit colors when `COLORTERM` is `truecolor` or `24bit` (Windows consoles and Windows Terminal always do), and falls back to 256 colors otherwise. It draws its squares with Unicode characters (`■`, `▪`). Windows Terminal, the VS Code terminal, iTerm2, GNOME Terminal, Konsole, Ghostty and Zed show them with no setup.

## Installing on Windows

```powershell
git clone https://github.com/LetermeFlorent/claude-statusline.git
cd claude-statusline
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

The script does four things:

1. It downloads `statusline-windows-x86_64.exe`, or `statusline-windows-arm64.exe` on an Arm processor, and `SHA256SUMS` from the latest release, and stops if the SHA-256 hash of the binary is missing from the list or differs from it. It first writes the new program next to the old one as `statusline.new.exe` and unblocks it (`Unblock-File`), then renames the previous `statusline.exe` to `statusline.exe.old-<timestamp>`, which allows the swap even during a Claude Code session, and renames the new file into `%USERPROFILE%\.claude\bin\statusline.exe`. If any of that fails, the previous program is put back and the script stops with a message, so a failed copy never leaves the account without a program. The `.old` copies are deleted when possible, along with those left by an automatic update.
2. It installs `statusline.json` in `%USERPROFILE%\.claude`, only when that file does not exist yet. The other accounts read this same file, unless they have their own in their folder.
3. It lists the accounts: `.claude`, every `.claude-compte*` folder in the home folder, and every `Dir` in `.claude-accounts.json` (the registry of [claude-account-menu](https://github.com/LetermeFlorent/claude-account-menu)), without duplicates or missing folders. An unreadable registry gives a warning.
4. For each account, it runs `statusline.exe --install` with `CLAUDE_CONFIG_DIR` set to the account folder. The command is rewritten in its portable form, `~/.claude/bin/statusline.exe`: any other command, such as a path from an old PC or an old user name, is replaced, and the script says so. The program also repairs the `statusLine` of the account's global file, `.claude.json` in its folder (and `%USERPROFILE%\.claude.json` for `.claude`), when it points to a statusline program that no longer exists; any other command there, such as `npx ccusage statusline`, is left alone. A failure is reported for that account and does not stop the others.

With `-Build`, the script skips the download and compiles the sources (`cargo build --release`, Rust required). This is also the way out when the download fails, or when the release has no `SHA256SUMS`.

## Installing on Linux or macOS

```sh
git clone https://github.com/LetermeFlorent/claude-statusline.git
cd claude-statusline
sh install.sh
```

The program goes to `~/.claude/bin/statusline`. When `cargo` is available, the script compiles the sources in the `target` folder of the repository. When `cargo` is missing, or the build fails (the script then says so), it downloads the binary of the latest release that matches `uname`, along with the `SHA256SUMS` list of the same release, and stops if the SHA-256 hash of the binary is missing from the list or differs from it. On any other system or architecture it stops with a message pointing to Rust.

The binary is first written as `statusline.new`, then renamed, so that a bar already running keeps the old file until it exits. The rest matches Windows: `statusline.json` installed if missing, then `--install` for `.claude`, every `.claude-compte*` and every `Dir` of the registry (stale paths are repaired the same way, `~/.claude.json` included). The registry is read with `grep` and `sed`, with no JSON parser, so it must keep the format claude-account-menu writes.

The scripts speak French or English by the same rule as the program: `STATUSLINE_LANG` first, then the display language on Windows, or `LANGUAGE`, `LC_ALL`, `LC_MESSAGES` and `LANG` elsewhere. Once the program is copied, they ask it for its language with `--language` and pass the answer to each `--install` through `STATUSLINE_LANG`, so the whole run speaks one language. A release older than this option has no `--language`: the scripts then keep their own choice.

## What --install does

`--install` edits the `settings.json` of the current account: the one in the `CLAUDE_CONFIG_DIR` folder when that variable is set, `~/.claude/settings.json` otherwise. It goes through these steps:

- A copy of the file is kept next to it, named `settings.json.bak-<timestamp>`, where the timestamp counts seconds since 1970 (`-1`, `-2` are added on a collision). The three newest copies are kept and older ones are deleted. There is no copy when the file did not exist.
- The `statusLine` block gets `"type": "command"` and `"command"`. When the program sits in the home folder, the command starts with `~`: `~/.claude/bin/statusline.exe` on Windows, `~/.claude/bin/statusline` elsewhere. It then works on every PC, so `settings.json` can be synced between machines. The full path, written with `/`, is used when the program is outside the home folder, when `HOME` names a folder other than `USERPROFILE` on Windows, or when the rest of its path holds a character other than ASCII letters, digits and `_./:@%+=,-`. A full path holding such a character, a space for instance, is put in single quotes for the shell. `refreshInterval` is set to 3 only when it is missing, and the other keys of the block are kept.
- The file is written back with a 2-space indent. Keys keep their order and values keep their exact text; a new `statusLine` block goes at the end.
- When no `statusline.json` is found, a short template is written in the account folder.

An empty `settings.json` counts as `{}`. A file that cannot be read, or that is not a JSON object (invalid JSON, comments, an array), is never touched: the command names the problem, changes nothing and exits with code 1.

## Checking

Close and restart Claude Code: the bar shows under the prompt. To try the program alone, without Claude Code:

```sh
~/.claude/bin/statusline --demo
```

On Windows, `& "$env:USERPROFILE\.claude\bin\statusline.exe" --demo`. Two lines of bars should appear. If nothing shows in Claude Code while this test works, open the `settings.json` of the account and check that it holds this block (`~/.claude/bin/statusline` outside Windows):

```json
"statusLine": {
  "type": "command",
  "command": "~/.claude/bin/statusline.exe",
  "refreshInterval": 3
}
```

`--version` prints the installed version, and `--theme` the detected background.

## Configuring the look

Everything is set in `~/.claude/statusline.json`, read again on every refresh, with no need to restart Claude Code. The bars follow the terminal background on their own, the text keeps the terminal's default color, and `statusline --theme` prints the detected background (`light` or `dark`) and where that choice came from. If detection gets it wrong, set `terminal_background` to `light` or `dark` in the file, or set the `STATUSLINE_BG` variable.

These are the settings most often changed:

- `bar_width` changes the number of squares (1 to 40);
- `segments` shows or hides `ctx`, `5h`, `7d` and `model`;
- `gradient` sets how strongly the 5h and 7d bars lighten, and `system.gradient` does the same for the second line;
- `thresholds` and `palettes` change the thresholds and hues of the quotas;
- in the `system` block, `metrics` picks the metrics and their order (`ram`, `io`, `cpu`), `"enabled": false` removes the second line, and `disk_path` picks the disk to watch on Windows and Linux.

An empty `metrics` list, or one with no known name, falls back to the three metrics: to hide them all, use `"enabled": false`. To use another settings file, set `STATUSLINE_CONFIG` to its path. The README lists every key with its default value.

Some terminals, mostly CJK locales and a few fonts, draw ambiguous-width characters such as `■` and `▪` two columns wide, which pushes the line out of line. This ASCII glyph set is one column wide everywhere, ready to copy into `~/.claude/statusline.json`:

```json
{ "glyphs": { "filled": "#", "empty": "-", "half": "+" } }
```

The marker before `ctx` is already plain ASCII (`~`), so it needs no change.

A mistake in the file does not show on the bar: invalid JSON brings every setting back to its default, and a misspelt key or a wrong value is skipped. After an edit, `statusline --check` lists each mistake with its line and column.

## Updating

Automatic updates are off by default. With `"auto_update": true` in `statusline.json`, the bar looks for a newer release on GitHub at most every 6 hours, installs it in the background and shows `update` during the swap, with no need to restart Claude Code. The "Automatic updates" section of the README has the details.

`statusline --update` runs the same update by hand and prints the outcome: already up to date, the version change (for example `2.1.0 -> v2.2.0`), or the reason it failed.

You can also run `git pull` in the cloned folder and run the install script again. The program is replaced even while a Claude Code session is running. `statusline.json` and the other settings are kept, and each `settings.json` gets a new `.bak-` copy.

## Uninstalling

`statusline --uninstall` removes the `statusLine` block of the current account, after a `settings.json.bak-<timestamp>` copy, and writes the file back the same way as `--install`. For another account, set `CLAUDE_CONFIG_DIR` to its folder before running the command. When the account has no `settings.json`, or no `statusLine` block in it, the command says which and changes nothing. A `settings.json` that cannot be read or is not a JSON object is left alone, with exit code 1.

Then delete the program, `~/.claude/bin/statusline` (`statusline.exe` on Windows), with any `statusline.new*` and `statusline.exe.old-*` an update left next to it. If you no longer want them, also delete `~/.claude/statusline.json`, any `statusline.json` in the account folders, the `settings.json.bak-*` copies, and the state folder:

- Windows: `%LOCALAPPDATA%\claude-statusline`;
- macOS: `~/Library/Application Support/claude-statusline`;
- Linux: `~/.local/state/claude-statusline`, or `$XDG_STATE_HOME/claude-statusline`.

## Troubleshooting

If the bar disappeared after a move to another PC or another user name, or after syncing `settings.json` between machines, the command in `statusLine` most likely points to a path on the other machine (`C:/Users/<old>/...`), written by a version older than 2.1.0, and Claude Code can no longer find the program. Run `git pull` in the cloned folder, then the install script again: it puts the program back, writes the portable command into the `settings.json` of each account and repairs the global `statusLine` in `~/.claude.json` when it points to a statusline program that no longer exists.

If `update` stays on screen, an update is still running: the bar shows it only while an update process holds its lock, and drops the marker of one that stopped halfway. `statusline --update` run by hand shows where it fails. The network must let `api.github.com` and `github.com` through.

If Windows blocks the program, run `Unblock-File "$env:USERPROFILE\.claude\bin\statusline.exe"`. The installer already does it, but unblocking can fail. The program has no digital signature, which can trigger a warning from SmartScreen or the antivirus.

If the `5h` and `7d` segments do not appear, Claude Code has not sent the quotas for this session yet, and the bar has no value less than 10 minutes old for this account: they appear after the first response. With claude-account-menu, the `SessionStart` hook fills them in at startup. If the 5-hour quota shows `--%`, the account has no 5-hour window open, meaning no message since the last reset.

Disk and CPU show `--%` on the first refresh, until there are two readings to compare, and after more than a minute without a refresh. They stay at `--%` on systems other than Windows, Linux and macOS. On Linux the disk also stays at `--%` when `disk_path` sits on a filesystem with no device behind it, such as ZFS, `/mnt/c` under WSL or the root of a container, on a machine with several disks: point `disk_path` to a folder on an ordinary disk partition, or remove `io` from `system.metrics`.

If the colors are wrong or the squares turn into question marks, the terminal does not handle 256 colors or the font lacks the `■` and `▪` characters. These characters can be changed in `glyphs`.

If the bar picks a dark palette on a light terminal under WSL, it most likely found no Windows profile, neither in `PATH` nor through the Windows Terminal package in `/mnt/c/Users`. Set `STATUSLINE_BG=light` in the shell profile, or `terminal_background` in `statusline.json`.

A change of auto-compact threshold can take 30 seconds to show in the bar.
