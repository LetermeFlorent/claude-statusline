#!/bin/sh
set -eu
here=$(cd "$(dirname "$0")" && pwd)
main="$HOME/.claude"
bin_dir="$main/bin"
repo="https://github.com/LetermeFlorent/claude-statusline"

mkdir -p "$bin_dir"
if command -v cargo >/dev/null 2>&1 && (cd "$here" && cargo build --release --quiet --target-dir "$here/target"); then
  cp "$here/target/release/statusline" "$bin_dir/statusline.new"
else
  command -v cargo >/dev/null 2>&1 && echo "compilation impossible, passage au binaire publie" >&2
  case "$(uname -s)-$(uname -m)" in
    Linux-x86_64) asset=statusline-linux-x86_64 ;;
    Linux-aarch64 | Linux-arm64) asset=statusline-linux-arm64 ;;
    Darwin-arm64) asset=statusline-macos-arm64 ;;
    Darwin-x86_64) asset=statusline-macos-x86_64 ;;
    *)
      echo "aucun binaire publie pour $(uname -s) $(uname -m) : installer Rust (https://rustup.rs) puis relancer" >&2
      exit 1
      ;;
  esac
  echo "telechargement de $asset"
  fetch() { curl -q -fsSL --proto =https --proto-redir =https "$repo/releases/latest/download/$1" -o "$2"; }
  sha256() { if command -v sha256sum >/dev/null 2>&1; then sha256sum; else shasum -a 256; fi < "$1" | cut -d ' ' -f 1; }
  fetch SHA256SUMS "$bin_dir/statusline.sums" && fetch "$asset" "$bin_dir/statusline.new" || {
    rm -f "$bin_dir/statusline.sums" "$bin_dir/statusline.new"
    echo "telechargement impossible : installer Rust (https://rustup.rs) puis relancer" >&2
    exit 1
  }
  expected=$(awk -v name="$asset" '$2 == name || $2 == "*" name { print $1 }' "$bin_dir/statusline.sums")
  rm -f "$bin_dir/statusline.sums"
  if [ -z "$expected" ] || [ "$(sha256 "$bin_dir/statusline.new")" != "$expected" ]; then
    rm -f "$bin_dir/statusline.new"
    echo "empreinte SHA-256 absente de SHA256SUMS ou differente : binaire refuse" >&2
    exit 1
  fi
fi

chmod 755 "$bin_dir/statusline.new"
mv -f "$bin_dir/statusline.new" "$bin_dir/statusline"
echo "programme copie dans $bin_dir/statusline"

if [ -f "$main/statusline.json" ]; then
  echo "statusline.json existe deja, conserve"
else
  cp "$here/statusline.json" "$main/statusline.json"
  echo "statusline.json installe"
fi

seen=""
install_for() {
  d=${1%/}
  [ -d "$d" ] || return 0
  case "|$seen|" in *"|$d|"*) return 0 ;; esac
  seen="$seen|$d"
  CLAUDE_CONFIG_DIR="$d" "$bin_dir/statusline" --install || echo "compte laisse tel quel : $d" >&2
}

install_for "$main"
for d in "$HOME"/.claude-compte*; do
  install_for "$d"
done
reg="$HOME/.claude-accounts.json"
if [ -f "$reg" ]; then
  grep -o '"Dir"[[:space:]]*:[[:space:]]*"[^"]*"' "$reg" | sed 's/.*"\([^"]*\)"$/\1/' | while IFS= read -r rel; do
    install_for "$HOME/$rel"
  done
fi
echo "termine, relancer Claude Code"
