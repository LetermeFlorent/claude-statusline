# claude-statusline

A status line for Claude Code on Windows, Linux and macOS. It takes two lines: the first follows the session (context, 5-hour and 7-day quotas, model and effort), the second follows the machine (RAM, disk activity, CPU).

```
ctx ■■▪■■■■■ 340k/1M | 5h ■■■■■■■■ 36% 3h29m | 7d ■■■■■■■■ 61% 3d16h | Opus 5:max
ram ■■■■■■■■ 11.3/15.3G | C: ▪■■■■■■■ 2% | cpu ▪■■■■■■■ 8%
```

It is a small Rust program with no dependencies. Claude Code runs it every 3 seconds through the `statusLine` setting: it reads the session JSON on standard input, writes the bar on standard output and exits after a little over a millisecond of work. On Windows the binary does without the C runtime and weighs about 220 KB. It can update itself from the GitHub releases, an option that is off by default.

## Install

The full guide, with checking, updating and uninstalling, is in [INSTALL.md](INSTALL.md).

```powershell
# Windows
git clone https://github.com/LetermeFlorent/claude-statusline.git
cd claude-statusline
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

```sh
# Linux, macOS
git clone https://github.com/LetermeFlorent/claude-statusline.git
cd claude-statusline
sh install.sh
```

Neither script needs the clone to install a released binary: download the script alone from the `main` branch and run it, as [INSTALL.md](INSTALL.md) shows for each system. The checksum check applies the same way, and a release without a `SHA256SUMS` is refused. Compiling, with `-Build` or through `cargo`, needs the sources. To remove the bar from every account, run `sh install.sh --uninstall` or `install.ps1 -Uninstall`.

Both scripts set up the bar in `.claude`, in every `.claude-compte*` folder and in every account listed by the multi-account menu ([claude-account-menu](https://github.com/LetermeFlorent/claude-account-menu)). `install.ps1` downloads the binary of the latest release and checks its SHA-256 hash, or compiles the sources with `-Build`. `install.sh` compiles when `cargo` is available, and downloads the binary of the latest release, checked the same way, when `cargo` is missing or the build fails.

## What the bar shows

The first line chains four segments separated by `|`, always in this order:

- `ctx`: how full the context is. The text gives the tokens used over the window size (`340k/1M`), the percentage alone when Claude Code does not send both, or `--%` with no data at all. The bar runs from light blue to deep blue, then turns orange from 70% and red from 90%, the same two hues as the machine metrics.
- `5h`: the 5-hour quota as a percentage, then the time left before it resets (`3h29m`, or `45m` under an hour). With no known reset time it shows `--` in that place.
- `7d`: the weekly quota, with the time before the reset in days and hours past one day (`3d16h`).
- the model, underlined, followed by `:` and the effort level. The effort letters are drawn in a gradient, with a hue of their own for `low`, `medium`, `high`, `xhigh` and `max`, and one shared hue for any other value. `auto` takes the hue of `low`, and `ultracode` shows as `ultracode(+workflows)`. The model name is cut before its first parenthesis: `Opus 4 (1M context)` becomes `Opus 4`.

The second line shows three machine metrics:

- `ram`: memory used over total memory, in GiB with one decimal. Used memory is the total minus the available memory. On Linux, a cgroup v2 memory limit below the memory of the machine, as in a container, becomes the total, and the used memory is then what the group uses minus its inactive file cache.
- the disk, labelled with its drive letter on Windows (`C:`) and `dsk` elsewhere: the share of time since the previous refresh during which the disk had at least one read or write in progress. macOS has no such counter, so there the bar adds up the time each request spent in progress, over every disk: requests served at the same time add up, and the bar can reach 100% on a disk that is not saturated.
- `cpu`: processor load since the previous refresh. On Linux, time spent waiting on I/O does not count as load.

Disk and CPU are differences between two readings, so they show `--%` on the very first run, after more than a minute without a refresh, after the watched disk changes (a new `disk_path`, or on macOS a disk plugged in or removed), and whenever their source does not answer. Two refreshes less than a second apart show the same percentage, and the second one keeps the earlier reading to compare with. They come from the system counters: `GetSystemTimes` and `IOCTL_DISK_PERFORMANCE` on Windows, `/proc/stat`, `/proc/diskstats` and `/proc/meminfo` on Linux, Mach and IOKit on macOS. The percentage is capped at 100.

Token counts are written as they are below 1000, then in `k`, `M` and `B`, rounded to the unit; a count that rounds up to 1000 moves to the next unit (999,500 reads `1M`).

### Bar colors

A quota bar takes its hue from its level: the normal hue under 30%, the warning hue from 30%, the critical hue from 70%. For the 5-hour quota that is violet, then mauve, then raspberry red; for the 7-day quota, green, then ochre, then red. The machine metrics keep their own hue (teal for RAM, khaki for the disk, blue for the CPU) up to 70%, turn orange above it, then red above 85%.

Within a bar, the squares get slightly lighter from left to right on a dark background, and darker on a light one. Filling is computed to the half square: the last filled square takes the small glyph (`▪`) only when it is half full, and a bar at 100% is made of full squares. Any value above zero fills at least one square, half full when the value is under half a square. The full and the empty squares share the `■` glyph by default and differ by color alone; `glyphs.empty` gives the empty ones another glyph. When the value reaches 100%, the number is drawn in the color of its bar. Labels and numbers are bold and stay in the default color of the terminal, so they read on any background; only the bars follow the detected background. `colors.ink_light` and `colors.ink_dark` color them.

### When parts are hidden

The `5h` and `7d` segments stay hidden as long as no quota is known for the account, neither in the session JSON nor in the state file from the last 10 minutes. The second line disappears when `system.enabled` is `false`, or when the `system.metrics` list holds no known name. Each segment can also be hidden through the `segments` setting; with all four hidden, the first line stays empty. During an update, `update` shows at the end of the second line, or on a line of its own when the second line is turned off.

When `COLUMNS` is set, the bar keeps each line within that many characters. It first drops the reset times of the `5h` and `7d` quotas, then the effort next to the model name, then narrows the bars one square at a time down to a single square. With the default settings the sample line takes 85 columns; with `bar_width` at `4`, it fits in 80 columns with every detail. Without `COLUMNS`, nothing is dropped.

## 5-hour and 7-day quotas

Claude Code sends the quotas in the session JSON (`rate_limits.five_hour` and `rate_limits.seven_day`, with `used_percentage` and `resets_at`), starting with the first API response. The bar shows them and stores them in the state file under the account's keys.

Before that first response, it reuses the values stored for the same account when they are less than 10 minutes old. A window whose reset time has passed, whether it comes from the session JSON or from the stored values, loses its percentage, which shows as `--%`, and keeps a reset time moved on by whole windows. That time is an estimate: a new 5-hour window only opens with the next message. The 5-hour quota stays at `--%` while the account has no 5-hour window open, that is before its first message.

The account is recognized by the name of its configuration folder: `CLAUDE_CONFIG_DIR` when it is set and not empty, `~/.claude` otherwise. The name loses its leading dot and goes to lowercase, and any character outside `a-z`, `0-9`, `.`, `_` and `-` becomes `_`. `.claude-compte2` gives `claude-compte2`, and an empty name gives `claude`. Several accounts open at the same time share the state file without mixing their values.

The `SessionStart` hook of [claude-account-menu](https://github.com/LetermeFlorent/claude-account-menu) writes the same keys when each session starts, from Anthropic's usage API, so the quotas show as soon as Claude Code opens.

## Auto-compact threshold

When an auto-compact threshold smaller than the context window is set, the `ctx` bar is measured against that threshold, the text becomes `used/threshold`, and a `~` followed by a space appears before `ctx`, as in `~ ctx ■■■■■■■■ 340k/400k`. The threshold is looked up in this order:

1. nothing at all when the `DISABLE_AUTO_COMPACT` environment variable is set to something other than empty, `0` or `false`;
2. the `CLAUDE_CODE_AUTO_COMPACT_WINDOW` environment variable of the process, if it is a positive integer;
3. otherwise three files, read one after the other: the `settings.json` of the account's configuration folder, then the project's `.claude/settings.json` and `.claude/settings.local.json`.

Each key is resolved on its own across the three files, the last file that sets it winning, and then the keys are tried in this order: `modelSettings.<model id>.autoCompactWindow` (the one `/autocompact` writes, for the model of the session), `env.CLAUDE_CODE_AUTO_COMPACT_WINDOW`, `autoCompactWindow`. `"autoCompactEnabled": false` in any file, unless a later file sets it back to `true`, removes the threshold, whatever the other keys and the environment variable say. A value is an integer or a string such as `200k` or `1M`; it is raised to 100000 or lowered to 1000000 when it falls outside that range, and then lowered to the context window of the model. The project folder is the `cwd` sent by Claude Code (the program's current folder when there is none) if it holds a `.claude/settings.json` or `.claude/settings.local.json`; otherwise it is the `workspace.project_dir` sent by Claude Code, and when neither holds one the `cwd` stays. A project folder on a network share is never probed, whichever field it comes from, and is left out, so only the account's file counts there: a UNC path or a network drive on Windows, an NFS, SMB, CIFS, Ceph, GlusterFS, AFS, SSHFS or rclone mount on Linux, or a mount that macOS does not mark as local. A file on a share that no longer answers would hold the bar for about 20 seconds on Windows, and with no end on a stuck NFS mount.

The files are read again at most every 30 seconds, and only when their size or date has changed, so a new setting can take 30 seconds to show. The threshold is not looked up while the `ctx` segment is hidden.

## Light or dark theme

The bar picks its colors from the terminal background. It takes the first answer in this order:

1. the `STATUSLINE_BG` variable;
2. the `terminal_background` setting in `statusline.json`;
3. the `COLORFGBG` variable that some terminals set: the number after the last `;` is the background color, and 7 or 9 and above mean a light background;
4. the settings of the host terminal, when it is recognized;
5. the system theme.

`STATUSLINE_BG` and `terminal_background` accept `light` or `clair`, `dark` or `sombre`, in any case. With no answer, the bar assumes a dark background.

### Recognized host terminals

The terminal is recognized by its environment variables, then its settings are read. A theme known by its name alone is light when the name holds `light`, `latte`, `dawn` or `day`, unless the word is part of `moonlight` or `twilight`, and dark when it holds `dark`, `night`, `black`, `moon`, `mocha` or `macchiato`. Any other name is looked up in the theme files of the terminal, as listed below, and counts as dark when none of them has it.

- VS Code and its forks (`TERM_PROGRAM=vscode`, `VSCODE_INJECTION` or `VSCODE_PID`). The user file is `User/settings.json` of Code, Code - Insiders, VSCodium, Cursor or Windsurf, in `%APPDATA%` on Windows, `~/Library/Application Support` on macOS, `$XDG_CONFIG_HOME` or `~/.config` on Linux. The product that started the terminal, found through `VSCODE_GIT_ASKPASS_NODE`, `VSCODE_GIT_ASKPASS_MAIN` or `VSCODE_CWD`, is read first. The `.vscode/settings.json` file of the project open in Claude Code, unless the project sits in a network folder, wins over the user file, key by key. A `terminal.background` color in `workbench.colorCustomizations`, set for the current theme (`"[Theme name]"`) or for all of them, decides through its luminance. Then `"window.autoDetectColorScheme": true` defers to the system theme. Otherwise `workbench.colorTheme` decides by its name, then through the `uiTheme` of the extension that brings it, in `~/.vscode/extensions` or the matching folder of a fork. With no theme set, the background is dark. Settings profiles are not read: only the default profile counts.
- Windows Terminal (`WT_SESSION`). Its files are the `settings.json` of each Microsoft Store package (stable, Preview, Canary) and the one of the unpackaged install. The bar reads the file that holds the profile named by `WT_PROFILE_ID`, and failing that the one saved last. The profile used is the one named by `WT_PROFILE_ID`, or else the default profile of that file. Its `background` color, or the one in `profiles.defaults`, decides through its luminance. Failing that, its color scheme decides: a scheme with two variants `{ "light", "dark" }` follows the application theme, given by the `theme` key of the file (`light`, `dark`, their `legacy` versions, or a custom theme from `themes` through its `window.applicationTheme`), and else the system; a custom scheme from `schemes` decides through the luminance of its background; among the built-in schemes, `One Half Light`, `Solarized Light` and `Tango Light` are light and the others dark. With no scheme set, it is `Campbell`, which is dark. Fragments, the profiles and schemes that other applications add, are not read.
- Zed (`TERM_PROGRAM=zed`), in `%APPDATA%\Zed\settings.json` on Windows and `~/.config/zed/settings.json` elsewhere. A string `theme` decides by its name, then through the `appearance` of the theme of that name, in the `themes` folder next to the settings or in an installed extension. An object `theme` follows its `mode` (`light`, `dark`), and `system`, or no setting, defers to the system.
- Ghostty (`TERM_PROGRAM=ghostty`). Its files are read in Ghostty's own order: `config.ghostty` then `config` in `~/.config/ghostty` (`$XDG_CONFIG_HOME/ghostty` when set), then on macOS the same two names in `~/Library/Application Support/com.mitchellh.ghostty`. A later value replaces an earlier one, an empty value clears it, and a file that sets neither key changes nothing. A `background` color decides through its luminance. Otherwise `theme` decides: a `light:...,dark:...` value follows the system, and a name decides by itself, then through the `background` of the theme file of that name, in `~/.config/ghostty/themes` or among the themes shipped with Ghostty (`GHOSTTY_RESOURCES_DIR`). With neither key, the background is dark.

Under WSL (`WSL_DISTRO_NAME`), VS Code and Windows Terminal keep their settings on the Windows side. The bar finds the Windows profile through the `/mnt/<drive>/Users/<name>` entry that Windows adds to `PATH`, or else through the folder of `/mnt/c/Users` that holds the Windows Terminal package, keeps that path in the state file and reads the files under it. When no profile turns up, the bar follows the system theme of the distribution, which is dark unless `GTK_THEME` or `gsettings` say otherwise: set `STATUSLINE_BG` to choose.

Other terminals go straight to the system theme, except on Windows, where the system theme describes apps and not terminals: a terminal the bar does not recognize counts as dark there. `COLORFGBG`, `terminal_background` and `STATUSLINE_BG` still decide first.

### System theme

Each system has its own source:

- Windows: the `AppsUseLightTheme` value in the user's registry (apps in light mode). A value that was never set means light mode, as it does for Windows itself.
- macOS: the `AppleInterfaceStyle` key of `~/Library/Preferences/.GlobalPreferences.plist`. `Dark` gives a dark background, and a missing key a light one.
- Linux: `GTK_THEME` (dark if it contains `dark`), then on KDE the `BackgroundNormal` color in the `[Colors:Window]` section of `kdeglobals`, then `gsettings`: `color-scheme` (`prefer-dark` or `prefer-light`), and failing that the name in `gtk-theme`. Each `gsettings` call gets one second to answer: one that hangs is stopped, and the background is then taken as dark.

When the terminal follows the system (VS Code with `window.autoDetectColorScheme`, a two-variant Windows Terminal scheme, Zed set to `system`, a Ghostty `light:...,dark:...` theme), the system theme settles the question.

The result of steps 4 and 5 is kept in the state file. It is worked out again as soon as one of the settings files listed above, the project's `.vscode/settings.json` included, appears, goes away, or changes size or date, when the bar runs under another Windows Terminal profile (`WT_PROFILE_ID`), or when the system theme changes, so a switch to light mode shows at the next refresh. A settings file kept as a symbolic link, in a dotfiles repository for instance, is followed to its target. Steps 1 to 3 are read on every run. `statusline --theme` prints the background in use, then a second line naming where the choice came from: `STATUSLINE_BG`, `terminal_background`, `COLORFGBG`, the host terminal, the system theme, or the dark default for an unknown terminal on Windows.

## Settings

The program looks for its settings file in this order:

1. the file named by `STATUSLINE_CONFIG`, even if it does not exist;
2. `statusline.json` in the `CLAUDE_CONFIG_DIR` folder, if it exists;
3. `~/.claude/statusline.json`.

It is read again on every refresh. Its JSON may hold `//` and `/* */` comments, trailing commas and a BOM. A missing or unreadable file leaves every value at its default, and so does invalid JSON, with no message on the bar. Colors are written `#RRGGBB` or `#RGB`, the `#` is optional, and an invalid color falls back to its default. The `statusline.json` in the repository repeats part of the defaults, and any key of the table can be added to it.

`statusline --check` reads the same file and lists its mistakes, each with its line and column: invalid JSON, text after the closing brace, unknown key, value of the wrong type, invalid color or unknown name. It exits with code 1 when it finds one.

| Key | Default | Effect |
|---|---|---|
| `terminal_background` | not set | `light`, `clair`, `dark` or `sombre` forces the background |
| `bar_width` | `8` | squares per bar, from 1 to 40 |
| `bold` | `true` | `false` drops the bold from labels and numbers |
| `glyphs.filled`, `glyphs.empty`, `glyphs.half` | `■`, `■`, `▪` | full square, empty square, half-filled last square |
| `auto_compact_marker` | `~` | auto-compact threshold marker, drawn before `ctx` and followed by a space; empty to hide it |
| `effort_style` | `gradient` | `flat`, in any case, draws the effort in a single hue |
| `segments.ctx`, `segments.5h`, `segments.7d`, `segments.model` | `true` | `false` hides the segment |
| `gradient.5h`, `gradient.7d` | `0.55`, `0.3` | how strongly the bar lightens, from 0 to 1 |
| `thresholds.ctx_warn`, `thresholds.ctx_hot` | `70`, `90` | percentages from which the context bar turns orange, then red (the blue gradient is kept below) |
| `thresholds.quota_warn`, `thresholds.quota_hot` | `30`, `70` | percentages from which a quota turns to warning, then critical |
| `thresholds.system_warn`, `thresholds.system_hot` | `70`, `85` | percentages above which a machine metric turns to warning, then critical |
| `palettes.5h.ok`, `.warn`, `.hot` | `#786EC8`, `#AA5AAF`, `#C83C5F` | hues of the 5-hour quota |
| `palettes.7d.ok`, `.warn`, `.hot` | `#538977`, `#B88143`, `#B95555` | hues of the 7-day quota |
| `colors.ink_light`, `colors.ink_dark` | not set | color of labels and numbers, on a light and on a dark background; unset, they keep the terminal's default color |
| `colors.empty_light`, `colors.empty_dark` | `#C4C8CE`, `#55595F` | empty squares |
| `colors.ctx_from_light`, `colors.ctx_to_light` | `#3D8BEB`, `#285AB4` | context gradient on a light background |
| `colors.ctx_from_dark`, `colors.ctx_to_dark` | `#96CDFF`, `#4682DC` | context gradient on a dark background |
| `system.enabled` | `true` | `false` removes the second line |
| `system.metrics` | `["ram", "io", "cpu"]` | metrics shown, in the order of the list: `ram`, `io` (or `dsk`) and `cpu`, in any case. A name listed twice counts once, and a list with no known name shows all three |
| `system.gradient` | `0.45` | how strongly the bars of the second line lighten |
| `system.disk_path` | `C:\` on Windows, `/` elsewhere | disk to watch |
| `system.colors.ram.ok`, `.dsk.ok`, `.cpu.ok` | `#1E787D`, `#697855`, `#4F7FC4` | normal hue of each metric |
| `system.colors.<metric>.warn`, `.hot` | `#BE823C`, `#C83C3C` | warning and critical hues |
| `auto_update` | `false` | `true` turns on the automatic update check |

Names in `system.metrics` are written in lowercase, and unknown names are ignored. An empty or missing list falls back to the three metrics. The disk color is set under `dsk`, even though the metric is called `io`.

`system.disk_path` is read differently on each system. On Windows only the first letter counts: `D:\Games` follows drive `D:`. On Linux it is the device that holds that path. Some filesystems, btrfs for one, give no device number of their own: the bar then finds the mount point of the path in `/proc/self/mountinfo` and follows the device mounted there. When that leads to no device, as on ZFS, on `/mnt/c` under WSL or at the root of a container, the bar falls back to the only physical disk of the machine, and shows `--%` when there are several. Under WSL, `/` is the virtual disk of the distribution, not a Windows drive. On macOS the setting is ignored and the bar adds up the activity of every disk.

## Command line

The program is installed in `~/.claude/bin`, as `statusline.exe` on Windows and `statusline` elsewhere. That folder is not on the PATH, so call the program by its full path, for example `~/.claude/bin/statusline --theme`. `--install` writes it into `settings.json` in that same form, with `~`, so the command works on every PC whatever the user name.

| Command | Effect |
|---|---|
| no argument | reads the Claude Code JSON on standard input, writes the bar and updates the state file. Run by hand in a terminal, it does not wait for input and draws a bar with no data. |
| `--demo [low\|warn\|hot]` | sample rendering without Claude Code (Opus 5, effort `max`, context 340k of 1M, 5h at 36%, 7d at 61%), with the real settings and theme. The disk and the cpu show sample values (12% and 18%) and the memory is the real one. `warn` and `hot` move the context, memory, disk and cpu bars across the warning and hot thresholds (disk 76% and 93%, cpu 78% and 96%, context 74% and 93%), to see their colors. The state file is left alone. |
| `--bench [n]` | renders the sample `n` times (200 by default) and prints the average time per rendering, then its breakdown: state, config, session, theme, compact, system, render, write. The first rendering starts from an empty state; each one then reads the state the previous one wrote, and writes it back only when it changed. Process startup is not counted. |
| `--theme` | prints the background in use, `light` or `dark`, and on a second line where that choice came from |
| `--language` | prints the language of the messages, `fr` or `en`; the install scripts ask for it |
| `--check` | lists the mistakes in `statusline.json` with their line and column, and exits with code 1 when there is one (see [Settings](#settings)) |
| `--install` | writes the `statusLine` block into the `settings.json` of the current account (see [INSTALL.md](INSTALL.md)) |
| `--uninstall` | removes that block |
| `--update` | installs the latest release when it is newer (see below) |
| `--version`, `-V` | prints `statusline` followed by the version number |
| `--help`, `-h` | prints the help |

Only the first argument is read, along with the count after `--bench`. The exit code is 0 on success, 1 when `--install`, `--uninstall` or `--update` fails, and 2 for a usage error: an unknown argument, printed back with the help, or a `--bench` count that is not a whole number above 0. `--update` has two codes of its own: 3 when the downloaded binary reports a version other than the release, and 4 when the old program could not be put back after a failed swap on Windows.

The help and the messages follow the system language: French on a system set to French, English on any other. `STATUSLINE_LANG` comes first, with `fr` or `en` in any case. A value that is neither is ignored. Otherwise, on Windows, the display language of the user account decides. On Linux and macOS the first of `LC_ALL`, `LC_MESSAGES` and `LANG` that holds a value decides, and French means a language code of `fr`, as in `fr_FR.UTF-8` or `fr_CA`; a locale of `C` or `POSIX` means English. On Linux, `LANGUAGE` is read before them and its first entry in French or English wins, so `de:fr` gives French. On macOS, when none of the three holds a value, the first language of the system preferences (`AppleLanguages`) that is French or English decides. The two install scripts follow the same rule, in the shell for the first messages. Once the program is copied they ask it for its language, which also covers `AppleLanguages` on macOS, then pass the answer to `--install` through `STATUSLINE_LANG`.

All text in and out is UTF-8: the session JSON is read as UTF-8, and anything the program writes to a pipe or a file is UTF-8, on every system. On a Windows console the text goes out through the wide-character console call, so the code page set with `chcp` changes nothing. An invalid byte in the input becomes U+FFFD instead of stopping the bar.

## Automatic updates

With `"auto_update": true` in `statusline.json`, rendering the bar starts `statusline --update` at most every 6 hours, in the background, with no window and at low priority, and returns without waiting for it. This only happens when the program runs from `~/.claude/bin` and a binary is published for the system. The time of the last check is kept in the state file, under the `update_at` key.

`--update` asks the GitHub API for the latest release, through `curl` (on Windows, the `curl.exe` of the system folder). When that release is newer, it downloads the binary for the system next to the program, under a name that carries its process id (`statusline.new-<pid>.exe`, or `statusline.new-<pid>`). It runs that copy with `--version`, which must print the version of the release, then with `--demo`, and only then puts it in place of the old one. On Windows the running program cannot be overwritten, but it can lose one of its names: the update first gives it a second one, `statusline.exe.old-<pid>`, then renames the new file over `statusline.exe` in a single step, so `statusline.exe` exists at every moment. Where hard links are not available, the old program is renamed away first, and put back if the new one cannot take its place.

Each update removes what earlier ones left behind: the `statusline.new*` and `statusline.exe.old-*` files next to the program, and the `.tmp` files older than a minute in the state folder.

During the download, then for 5 seconds after the swap, the bar shows `update`. The next refresh starts the new program, with no need to restart Claude Code. A failure (network down, missing binary, wrong version, swap refused) leaves the old program in place until the next check.

The update holds a lock on `update.lock`, in the folder of the state file, for its whole run: an open handle on Windows, `flock` on Unix. The system releases it when the process ends, even when it is killed. A second update waits at most 2 seconds for it, then says that an update is already running and stops. The bar shows `update` only while the `updating` marker exists and the lock is held. It checks the lock without waiting, and removes a marker that a stopped update left behind. The automatic check is off by default because nothing signs the downloaded binary yet. `statusline --update` still works by hand.

Debug builds, and only those, read a loopback address from `STATUSLINE_UPDATE_SERVER` in place of GitHub. The tests use it to run `--update` against a local server.

## State file

The counters, the quotas and the detected theme live in a single file, shared by every account:

- Windows: `%LOCALAPPDATA%\claude-statusline\state`;
- macOS: `~/Library/Application Support/claude-statusline/state`;
- Linux: `$XDG_STATE_HOME/claude-statusline/state`, or else `~/.local/state/claude-statusline/state`.

It is a text file with one `key value` line per entry. It holds the quotas of each account (`q_at@<account>`, `q5_at@<account>`, `q5_pct@<account>`, `q7_at@<account>`, `q7_pct@<account>`), the theme (`bg_host`, `bg_stamp`), the auto-compact threshold (one `compact@<fingerprint of the settings files>` entry per project, the 4 most recently checked kept, each holding the check time, the files stamp and the window; the single entry keys `compact_window`, `compact_key`, `compact_stamp` and `compact_at` of older versions are removed), the last disk and CPU readings (`io_source`, `io_clock`, `io_value`, `cpu_source`, `cpu_clock`, `cpu_value`) and the time of the last update check (`update_at`). Unknown keys are kept. The version 1 keys (`bg_at`, and the quotas without an account suffix) are removed on the first run.

A refresh writes the file again only when its content changes. That is nearly every refresh a second or more after the previous one, since the disk and CPU readings and the `q_at` time of the account move each time; refreshes within the same second mostly write nothing. Each write goes through a temporary file that is then renamed, so a reader never finds it half written. Bars often refresh at the same moment, one per open session. Before writing, each one takes the `state.lock` file next to the state, then reads the file again and applies only the keys it changed itself, so the quotas another session wrote in between are kept. A bar that finds the lock taken leaves its changes in a `state.<process id>.changes` file next to the state, one `key value` line per key it set and a bare `key` line per key it removed, then waits half a second at most. The next bar to hold the lock applies every such file, saves, and deletes them, and every bar reads them along with the state, so a quota written while the lock was taken shows at once and is never lost. On Windows the rename goes through even while another bar is reading the file. When it still fails, because another program holds the file open without sharing it, that refresh writes nothing and the previous file stays whole. A tool that edits the file should follow the same steps: lock `state.lock` (`LockFileEx` on Windows, `flock` on Linux and macOS), read the file, change its own keys, then write a temporary file in the same folder and rename it over `state`, leaving the `.changes` files alone. With no known home folder, nothing is written: the home folder is `USERPROFILE`, then `HOME`, on Windows, and `HOME`, then the user's entry in the password database, on Linux and macOS. `--bench` writes a `state.bench` file next to it and deletes it at the end.

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

```sh
cargo build --release
```

The binary lands in `target/release`. On Windows, `.cargo/config.toml` links the program with `rust-lld` and without the C runtime, so nothing but Rust is needed. The `x86_64-pc-windows-msvc` and `aarch64-pc-windows-msvc` targets are supported there, and the few C runtime functions the compiler still calls (`memcpy`, `memset`, `__chkstk` and some others) are written in assembly, one file per target in `src/os`. The same file turns on CET protection (`/CETCOMPAT`) on x86_64, replaces the timestamp of the binary with a hash of its content (`/Brepro`) and produces no `.pdb`: two builds of the same code give the same file, whatever the folder. On macOS it passes `-S` to the linker, which then writes no object file path: the UUID of the binary, computed over its content, no longer depends on the build folder. The release profile turns on `lto` and `panic = "abort"`, and strips the symbols.

The tests (`cargo test`) run as they are on Linux and macOS. On Windows they need the Visual Studio C runtime and a `RUSTFLAGS` variable that replaces the link options of the config file, as CI does (`RUSTFLAGS=-Cdebuginfo=0`).

## CI and releases

CI runs on every push to `main`, on every `v*` tag and on every pull request. On each of the six targets it checks the formatting, runs `clippy` and the unit tests, builds in release mode, then tries the binary: version, theme, sample rendering, reading a session JSON, even one holding an invalid UTF-8 byte, benchmark, install then uninstall, install from `~/.claude/bin` with its `~` command, plain text under `NO_COLOR` and `TERM=dumb`, and refusal of an unknown argument or of a bad `--bench` count. On Linux it also reads the memory shown inside a systemd scope capped at 512 MB. It feeds two full sample inputs and reads the measured values from the output once its colors are stripped: `ram` as N.N/N.NG, `cpu` as NN%, and on Linux and macOS a total memory equal to `free -b` or `sysctl hw.memsize`. It runs `--help` and `--demo` in French and in English through `STATUSLINE_LANG`, and through `LANG` on Linux and macOS, and checks that the help text is in the right language. It then runs the installer twice in an empty home folder, `install.sh` on Linux and macOS and `install.ps1 -Build` under both `powershell.exe` and `pwsh` on Windows, in a home folder that holds a `.claude-compte2` account, and compares each `settings.json` with the expected text. On Linux and macOS it also runs `install.sh` with `cargo` off the path, which downloads the latest release and checks its SHA-256 hash. It also checks the ASLR and NX flags of both Windows binaries and the CET flag of the x86_64 one, and that the Linux x86_64 binary is position-independent. A weekly run, which blocks nothing, builds and tests the sources on the stable and beta toolchains of Rust, on Linux and Windows, so that a new compiler breaking the build shows up before a release does.

A `v*` tag publishes a release with the binaries below. The tag must match the version in `Cargo.toml` (`v2.1.0` for `2.1.0`), or CI fails before building. The automatic update compares that version with its own.

| File | System |
|---|---|
| `statusline-linux-x86_64` | Linux x86_64, static binary (musl) |
| `statusline-linux-arm64` | Linux arm64, static binary (musl) |
| `statusline-macos-arm64` | macOS on Apple silicon |
| `statusline-macos-x86_64` | macOS on Intel |
| `statusline-windows-x86_64.exe` | Windows x86_64 |
| `statusline-windows-arm64.exe` | Windows arm64 |

CI builds with Rust 1.99.1, pinned by the `RUSTUP_TOOLCHAIN` variable of the workflow. It rebuilds every binary from a copy of the sources in another folder, and fails if the result differs by a single byte. Each release also publishes `SHA256SUMS`, which gives the SHA-256 hash of each binary, and a build provenance attestation that ties each file to the workflow that built it. To check a downloaded binary with GitHub CLI:

```sh
gh attestation verify statusline-linux-x86_64 --repo LetermeFlorent/claude-statusline
```

On Linux, `sha256sum -c SHA256SUMS --ignore-missing` also compares the binaries in the folder with the list.

`install.ps1` and `install.sh` download the file for their system from the latest release, along with `SHA256SUMS`, and refuse a binary whose hash is missing from the list or differs from it.

## Limits

The Linux arm64 binary always loads at the same address, because Rust cannot yet build a static position-independent executable for the `aarch64-unknown-linux-musl` target. On other Unix systems (BSD, for example) the bar works without machine metrics, system theme or automatic updates: `--%` everywhere on the second line, and a dark background unless one is set. Automatic updates need `curl`, which ships with macOS and with Windows 10 since version 1803, and has to be installed on some Linux systems.

Colors are sent as 24-bit when `COLORTERM` is `truecolor` or `24bit`, and always on Windows consoles and Windows Terminal. Anywhere else the bar falls back to the 256-color palette, picking the nearest color of the 6x6x6 cube or of the gray ramp, so the hues stay close and no 24-bit sequence reaches a terminal that would misread it. When `NO_COLOR` is set or `TERM` is `dumb`, the bar comes out with no escape sequence at all, so without colors or text styles, and with the same glyphs and values. Background detection through the host terminal covers VS Code and its forks, Windows Terminal, Zed and Ghostty. Other terminals go through `COLORFGBG` or the system theme, and `STATUSLINE_BG` can force the choice.
