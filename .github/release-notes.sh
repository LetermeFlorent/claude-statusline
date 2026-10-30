#!/bin/bash
set -eu

tag="${1:-}"
case "$tag" in
  v[0-9]*) ;;
  *) echo "usage: release-notes.sh <tag>, for example v2.1.2" >&2; exit 2 ;;
esac
case "$tag" in
  *[!0-9A-Za-z.+-]*) echo "release-notes.sh: unexpected characters in the tag" >&2; exit 2 ;;
esac

version="${tag#v}"
changelog="$(dirname "$0")/../CHANGELOG.md"

section() {
  [ -f "$changelog" ] || return 0
  tr -d '\r' < "$changelog" | awk -v version="$version" '
    substr($0, 1, 3) == "## " {
      if (active) exit
      title = substr($0, 4)
      active = (title == version || index(title, version " ") == 1)
      next
    }
    active { lines[++count] = $0 }
    END {
      first = 1
      while (first <= count && lines[first] == "") first++
      while (count >= first && lines[count] == "") count--
      for (i = first; i <= count; i++) print lines[i]
    }'
}

notes=$(section)

sed "s/@TAG@/$tag/g" <<'EOF'
Status line binaries for Windows, Linux and macOS. Pick the file for your system:

| System | File |
|---|---|
| Windows x64 | `statusline-windows-x86_64.exe` |
| Windows ARM64 | `statusline-windows-arm64.exe` |
| Linux x64 | `statusline-linux-x86_64` |
| Linux ARM64 | `statusline-linux-arm64` |
| macOS Apple silicon | `statusline-macos-arm64` |
| macOS Intel | `statusline-macos-x86_64` |

## Install

You do not have to download a file by hand: the install script fetches the one for your system, checks its SHA-256 hash and sets up every Claude Code account. On Windows, in PowerShell:

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/LetermeFlorent/claude-statusline/main/install.ps1 -OutFile install.ps1
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

On Linux and macOS:

```sh
curl -fsSL https://raw.githubusercontent.com/LetermeFlorent/claude-statusline/main/install.sh -o install.sh
sh install.sh
```

EOF

if [ -n "$notes" ]; then
  printf '## What changed\n\n%s\n\n' "$notes"
fi

sed "s/@TAG@/$tag/g" <<'EOF'
## Verify a download

SHA256SUMS lists the SHA-256 hash of every file above. Put it in the folder of the files you downloaded and run `sha256sum -c SHA256SUMS --ignore-missing`. Each file also carries a build provenance attestation: `gh attestation verify <file> --repo LetermeFlorent/claude-statusline`.

The full history is in [CHANGELOG.md](https://github.com/LetermeFlorent/claude-statusline/blob/@TAG@/CHANGELOG.md).
EOF
