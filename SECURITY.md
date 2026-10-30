# Security policy

## Reporting a vulnerability

Please do not open a public issue for a security problem. Use GitHub's private vulnerability reporting instead: open the Security tab of this repository, choose "Report a vulnerability", and fill in the form. Only the maintainer sees the report.

A useful report says:

- which version is affected (the output of `statusline --version`) and on which system and version (Windows, Linux or macOS);
- what an attacker can do, and what they need first (a local account, a crafted file, a hostile project folder, a network position);
- the steps or the input that reproduce it, as short as you can make them;
- what you expected to happen instead, and a fix if you already have one.

Please do not include real tokens, account files or the contents of your `~/.claude` folder. A redacted sample is enough.

## What to expect

Reports are read by one person, so replies are not instant. You will get an answer on whether the report is accepted, and a fix lands in a new release with the problem described in CHANGELOG.md once it can be used safely.

## Supported versions

Only the latest release gets fixes.

## Automatic updates

The program can replace itself with `--update`, but automatic updates are off by default. Release binaries are not signed yet, and updates stay opt-in until they are.
