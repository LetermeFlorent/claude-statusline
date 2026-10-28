#!/bin/sh
set -eu
here=$(cd "$(dirname "$0")" && pwd)
main="$HOME/.claude"
bin_dir="$main/bin"
repo="https://github.com/LetermeFlorent/claude-statusline"

language=en
case "${LC_ALL:-${LC_MESSAGES:-${LANG:-}}}" in
  fr | fr_* | fr.* | fr@*) language=fr ;;
esac

show() {
  case "$language.$1" in
    en.build_failed) text="build failed, using the released binary" ;;
    fr.build_failed) text="compilation impossible, passage au binaire publié" ;;
    en.no_asset) text="no released binary for $2 $3: install Rust (https://rustup.rs) and run this script again" ;;
    fr.no_asset) text="aucun binaire publié pour $2 $3 : installez Rust (https://rustup.rs) puis relancez" ;;
    en.downloading) text="downloading $2" ;;
    fr.downloading) text="téléchargement de $2" ;;
    en.download_failed) text="download failed: install Rust (https://rustup.rs) and run this script again" ;;
    fr.download_failed) text="téléchargement impossible : installez Rust (https://rustup.rs) puis relancez" ;;
    en.checksum) text="SHA-256 hash missing from SHA256SUMS or different: binary refused" ;;
    fr.checksum) text="empreinte SHA-256 absente de SHA256SUMS ou différente : binaire refusé" ;;
    en.copied) text="program copied to $2" ;;
    fr.copied) text="programme copié dans $2" ;;
    en.config_kept) text="statusline.json already exists, kept" ;;
    fr.config_kept) text="statusline.json existe déjà, conservé" ;;
    en.config_installed) text="statusline.json installed" ;;
    fr.config_installed) text="statusline.json installé" ;;
    en.account_kept) text="account left as is: $2" ;;
    fr.account_kept) text="compte laissé tel quel : $2" ;;
    en.done) text="done, restart Claude Code" ;;
    fr.done) text="terminé, relancez Claude Code" ;;
  esac
  printf '%s\n' "$text"
}

mkdir -p "$bin_dir"
if command -v cargo >/dev/null 2>&1 && (cd "$here" && cargo build --release --quiet --target-dir "$here/target"); then
  cp "$here/target/release/statusline" "$bin_dir/statusline.new"
else
  command -v cargo >/dev/null 2>&1 && show build_failed >&2
  case "$(uname -s)-$(uname -m)" in
    Linux-x86_64) asset=statusline-linux-x86_64 ;;
    Linux-aarch64 | Linux-arm64) asset=statusline-linux-arm64 ;;
    Darwin-arm64) asset=statusline-macos-arm64 ;;
    Darwin-x86_64) asset=statusline-macos-x86_64 ;;
    *)
      show no_asset "$(uname -s)" "$(uname -m)" >&2
      exit 1
      ;;
  esac
  show downloading "$asset"
  fetch() { curl -q -fsSL --proto =https --proto-redir =https "$repo/releases/latest/download/$1" -o "$2"; }
  sha256() { if command -v sha256sum >/dev/null 2>&1; then sha256sum; else shasum -a 256; fi < "$1" | cut -d ' ' -f 1; }
  fetch SHA256SUMS "$bin_dir/statusline.sums" && fetch "$asset" "$bin_dir/statusline.new" || {
    rm -f "$bin_dir/statusline.sums" "$bin_dir/statusline.new"
    show download_failed >&2
    exit 1
  }
  expected=$(awk -v name="$asset" '$2 == name || $2 == "*" name { print $1 }' "$bin_dir/statusline.sums")
  rm -f "$bin_dir/statusline.sums"
  if [ -z "$expected" ] || [ "$(sha256 "$bin_dir/statusline.new")" != "$expected" ]; then
    rm -f "$bin_dir/statusline.new"
    show checksum >&2
    exit 1
  fi
fi

chmod 755 "$bin_dir/statusline.new"
mv -f "$bin_dir/statusline.new" "$bin_dir/statusline"
show copied "$bin_dir/statusline"

if [ -f "$main/statusline.json" ]; then
  show config_kept
else
  cp "$here/statusline.json" "$main/statusline.json"
  show config_installed
fi

seen=""
install_for() {
  d=${1%/}
  [ -d "$d" ] || return 0
  case "|$seen|" in *"|$d|"*) return 0 ;; esac
  seen="$seen|$d"
  CLAUDE_CONFIG_DIR="$d" "$bin_dir/statusline" --install || show account_kept "$d" >&2
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
show done
