# Technical reference

This page holds the details that the [README](../README.md) only summarizes: how the host terminal is recognized, the format of the state file, every environment variable, the build and the CI checks.

## Recognized host terminals

The terminal is recognized by its environment variables, then its settings are read. A theme known by its name alone is light when the name holds `light`, `latte`, `dawn` or `day`, unless the word is part of `moonlight` or `twilight`, and dark when it holds `dark`, `night`, `black`, `moon`, `mocha` or `macchiato`. Any other name is looked up in the theme files of the terminal, as listed below, and counts as dark when none of them has it.

- VS Code and its forks (`TERM_PROGRAM=vscode`, `VSCODE_INJECTION` or `VSCODE_PID`). The user file is `User/settings.json` of Code, Code - Insiders, VSCodium, Cursor or Windsurf, in `%APPDATA%` on Windows, `~/Library/Application Support` on macOS, `$XDG_CONFIG_HOME` or `~/.config` on Linux. The product that started the terminal, found through `VSCODE_GIT_ASKPASS_NODE`, `VSCODE_GIT_ASKPASS_MAIN` or `VSCODE_CWD`, is read first. The `.vscode/settings.json` file of the project open in Claude Code, unless the project sits in a network folder, wins over the user file, key by key. A `terminal.background` color in `workbench.colorCustomizations`, set for the current theme (`"[Theme name]"`) or for all of them, decides through its luminance. Then `"window.autoDetectColorScheme": true` defers to the system theme. Otherwise `workbench.colorTheme` decides by its name, then through the `uiTheme` of the extension that brings it, in `~/.vscode/extensions` or the matching folder of a fork. With no theme set, the background is dark. Settings profiles are not read: only the default profile counts.
- Windows Terminal (`WT_SESSION`). Its files are the `settings.json` of each Microsoft Store package (stable, Preview, Canary) and the one of the unpackaged install. The bar reads the file that holds the profile named by `WT_PROFILE_ID`, and failing that the one saved last. The profile used is the one named by `WT_PROFILE_ID`, or else the default profile of that file. Its `background` color, or the one in `profiles.defaults`, decides through its luminance. Failing that, its color scheme decides: a scheme with two variants `{ "light", "dark" }` follows the application theme, given by the `theme` key of the file (`light`, `dark`, their `legacy` versions, or a custom theme from `themes` through its `window.applicationTheme`), and else the system; a custom scheme from `schemes` decides through the luminance of its background; among the built-in schemes, `One Half Light`, `Solarized Light` and `Tango Light` are light and the others dark. With no scheme set, it is `Campbell`, which is dark. Fragments, the profiles and schemes that other applications add, are not read.
- Zed (`TERM_PROGRAM=zed`), in `%APPDATA%\Zed\settings.json` on Windows and `~/.config/zed/settings.json` elsewhere. A string `theme` decides by its name, then through the `appearance` of the theme of that name, in the `themes` folder next to the settings or in an installed extension. An object `theme` follows its `mode` (`light`, `dark`), and `system`, or no setting, defers to the system.
- Ghostty (`TERM_PROGRAM=ghostty`). Its files are read in Ghostty's own order: `config.ghostty` then `config` in `~/.config/ghostty` (`$XDG_CONFIG_HOME/ghostty` when set), then on macOS the same two names in `~/Library/Application Support/com.mitchellh.ghostty`. A later value replaces an earlier one, an empty value clears it, and a file that sets neither key changes nothing. A `background` color decides through its luminance. Otherwise `theme` decides: a `light:...,dark:...` value follows the system, and a name decides by itself, then through the `background` of the theme file of that name, in `~/.config/ghostty/themes` or among the themes shipped with Ghostty (`GHOSTTY_RESOURCES_DIR`). With neither key, the background is dark.

Under WSL (`WSL_DISTRO_NAME`), VS Code and Windows Terminal keep their settings on the Windows side. The bar finds the Windows profile through the `/mnt/<drive>/Users/<name>` entry that Windows adds to `PATH`, or else through the folder of `/mnt/c/Users` that holds the Windows Terminal package, keeps that path in the state file and reads the files under it. When no profile turns up, the bar follows the system theme of the distribution, which is dark unless `GTK_THEME` or `gsettings` say otherwise: set `STATUSLINE_BG` to choose.

## System theme

Each system has its own source:

- Windows: the `AppsUseLightTheme` value in the user's registry (apps in light mode). A value that was never set means light mode, as it does for Windows itself.
- macOS: the `AppleInterfaceStyle` key of `~/Library/Preferences/.GlobalPreferences.plist`. `Dark` gives a dark background, and a missing key a light one.
- Linux: `GTK_THEME` (dark if it contains `dark`), then on KDE the `BackgroundNormal` color in the `[Colors:Window]` section of `kdeglobals`, then `gsettings`: `color-scheme` (`prefer-dark` or `prefer-light`), and failing that the name in `gtk-theme`. Each `gsettings` call gets one second to answer: one that hangs is stopped, and the background is then taken as dark.

When the terminal follows the system (VS Code with `window.autoDetectColorScheme`, a two-variant Windows Terminal scheme, Zed set to `system`, a Ghostty `light:...,dark:...` theme), the system theme settles the question.

## State file

The counters, the quotas and the detected theme live in a single file, shared by every account:

- Windows: `%LOCALAPPDATA%\claude-statusline\state`;
- macOS: `~/Library/Application Support/claude-statusline/state`;
- Linux: `$XDG_STATE_HOME/claude-statusline/state`, or else `~/.local/state/claude-statusline/state`.

It is a text file in UTF-8. A byte order mark at its start is ignored, and an invalid byte reads as U+FFFD. Each line is one entry, `key value`: the key runs to the first space and the value is the rest of the line, spaces included. A key holds no whitespace and a value no line break, and a line without a space is skipped. The program writes the lines sorted by key, each ended by a line feed, and reads a carriage return before the line feed too. Times are seconds since 1970 and percentages are numbers from 0 to 100. The keys are:

- `q_at@<account>`: when the quotas of the account were last stored, in whole seconds. Stored quotas more than 10 minutes old are not used.
- `q5_at@<account>` and `q7_at@<account>`: reset time of the 5-hour and of the 7-day window, as Claude Code sends it.
- `q5_pct@<account>` and `q7_pct@<account>`: percentage of each window used, as Claude Code sends it.
- `bg@<fingerprint of the terminal settings>`: the detected theme, one entry per terminal and project, the 4 most recently used kept, holding the time, `light` or `dark`, and the source of the choice.
- `compact@<fingerprint of the settings files>`: the auto-compact threshold, one entry per project, the 4 most recently checked kept, holding the check time, the files stamp and the window.
- `io_source`, `io_clock`, `io_value`, `io_at`, `io_pct`, and the same five under the prefix `cpu_`: the last disk and CPU readings. `_clock` and `_value` are the two raw counters, elapsed time and busy time, in the unit of the system source; `_source` tells what was read, `_at` is the time of the reading, with decimals, and `_pct` the percentage worked out from the last two readings.
- `update_at`: the time of the last update check.

Unknown keys are kept. Version 1 left keys that are no longer read: `bg_at`, `bg_host`, `bg_source` and `bg_stamp`, `compact_window`, `compact_key`, `compact_stamp` and `compact_at`, and the quotas without an account suffix (`q_at`, `q5_at`, `q5_pct`, `q7_at`, `q7_pct`). The theme keys go the next time the theme is looked up, the compact keys the next time the threshold is, and the quota keys the first time Claude Code sends quotas, so an old file keeps them until then.

A refresh writes the file again only when its content changes. That is nearly every refresh a second or more after the previous one, since the disk and CPU readings and the `q_at` time of the account move each time; refreshes within the same second mostly write nothing. Each write goes through a temporary file that is then renamed, so a reader never finds it half written. Bars often refresh at the same moment, one per open session. Before writing, each one takes the `state.lock` file next to the state, then reads the file again and applies only the keys it changed itself, so the quotas another session wrote in between are kept. A bar that finds the lock taken leaves its changes in a `state.<process id>.changes` file next to the state, one `key value` line per key it set and a bare `key` line per key it removed, then waits half a second at most. The next bar to hold the lock applies every such file, saves, and deletes them, and every bar reads them along with the state, so a quota written while the lock was taken shows at once and is never lost. On Windows the rename goes through even while another bar is reading the file. When it still fails, because another program holds the file open without sharing it, that refresh writes nothing and the previous file stays whole. A tool that edits the file should follow the same steps: lock `state.lock` (`LockFileEx` on Windows, `flock` on Linux and macOS), read the file, change its own keys, then write a temporary file in the same folder and rename it over `state`, leaving the `.changes` files alone. With no known home folder, nothing is written: the home folder is `USERPROFILE`, then `HOME`, on Windows, and `HOME`, then the user's entry in the password database, on Linux and macOS. `--bench` writes a copy of its own, `state-bench-<process id>` with its `.lock` file, next to the state and deletes both at the end.

## Environment variables

The program reads the following variables, all of them optional:

| Variable | Use |
|---|---|
| `STATUSLINE_BG` | forces the background: `light`, `clair`, `dark` or `sombre` |
| `STATUSLINE_CONFIG` | path of the settings file |
| `STATUSLINE_LANG` | forces the language of the help and the messages: `fr` or `en` |
| `CLAUDE_CONFIG_DIR` | account folder: settings, auto-compact threshold, quota keys, target of `--install` |
| `CLAUDE_CODE_AUTO_COMPACT_WINDOW` | auto-compact threshold |
| `COLORFGBG` | terminal colors, for the background |
| `COLORTERM` | `truecolor` or `24bit` keeps 24-bit colors; anything else, or nothing, switches to 256 colors |
| `COLUMNS` | width of the terminal: the line drops details to fit in it |
| `NO_COLOR`, `TERM` | plain text, when `NO_COLOR` is set or `TERM` is `dumb` |
| `TERM_PROGRAM`, `VSCODE_INJECTION`, `VSCODE_PID`, `WT_SESSION` | host terminal detection |
| `VSCODE_GIT_ASKPASS_NODE`, `VSCODE_GIT_ASKPASS_MAIN`, `VSCODE_CWD` | which of VS Code and its forks started the terminal |
| `WT_PROFILE_ID` | current Windows Terminal profile |
| `WSL_DISTRO_NAME`, `PATH` | under WSL, where the Windows profile is |
| `GHOSTTY_RESOURCES_DIR` | themes shipped with Ghostty |
| `GTK_THEME`, `XDG_CURRENT_DESKTOP` | system theme on Linux |
| `HOME`, `USERPROFILE`, `LOCALAPPDATA`, `APPDATA`, `XDG_STATE_HOME`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME` | where files are |

An empty variable counts as unset.

## Building

The binary lands in `target/release`. On Windows, `.cargo/config.toml` links the program with `rust-lld` and without the C runtime, so nothing but Rust is needed. The `x86_64-pc-windows-msvc` and `aarch64-pc-windows-msvc` targets are supported there, and the few C runtime functions the compiler still calls (`memcpy`, `memset`, `__chkstk` and some others) are written in assembly, one file per target in `src/os`. The same file turns on CET protection (`/CETCOMPAT`) on x86_64, replaces the timestamp of the binary with a hash of its content (`/Brepro`) and produces no `.pdb`: two builds of the same code give the same file, whatever the folder. On macOS it passes `-S` to the linker, which then writes no object file path: the UUID of the binary, computed over its content, no longer depends on the build folder. The release profile turns on `lto` and `panic = "abort"`, and strips the symbols.

The tests (`cargo test`) run as they are on Linux and macOS. On Windows they need the Visual Studio C runtime and a `RUSTFLAGS` variable that replaces the link options of the config file, as CI does (`RUSTFLAGS=-Cdebuginfo=0`).

## CI

CI runs on every push to `main`, on every `v*` tag and on every pull request. On each of the six targets it checks the formatting, runs `clippy` and the unit tests, builds in release mode, then tries the binary: version, theme, sample rendering, reading a session JSON, even one holding an invalid UTF-8 byte, benchmark, install then uninstall, install from `~/.claude/bin` with its `~` command, plain text under `NO_COLOR` and `TERM=dumb`, and refusal of an unknown argument or of a bad `--bench` count. On Linux it also reads the memory shown inside a systemd scope capped at 512 MB. It feeds two full sample inputs and reads the measured values from the output once its colors are stripped: `ram` as N.N/N.NG, `cpu` as NN%, and on Linux and macOS a total memory equal to `free -b` or `sysctl hw.memsize`. It runs `--help` and `--demo` in French and in English through `STATUSLINE_LANG`, and through `LANG` on Linux and macOS, and checks that the help text is in the right language. It then runs the installer twice in an empty home folder, `install.sh` on Linux and macOS and `install.ps1 -Build` under both `powershell.exe` and `pwsh` on Windows, in a home folder that holds a `.claude-compte2` account, and compares each `settings.json` with the expected text. On Linux and macOS it also runs `install.sh` with `cargo` off the path, which downloads the latest release and checks its SHA-256 hash, or, while that release publishes no `SHA256SUMS`, checks that the script refuses it and leaves the account untouched. It also checks the ASLR and NX flags of both Windows binaries and the CET flag of the x86_64 one, and that the Linux x86_64 binary is position-independent. A weekly run, which blocks nothing, builds and tests the sources on the stable and beta toolchains of Rust, on Linux and Windows, so that a new compiler breaking the build shows up before a release does.
