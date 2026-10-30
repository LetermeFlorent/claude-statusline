#!/bin/sh
set -eu
here=$(cd "$(dirname "$0")" && pwd)
main="$HOME/.claude"
bin_dir="$main/bin"
repo="https://github.com/LetermeFlorent/claude-statusline"
release_key='ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAc/fOxJG/NBMFcBqV6k6pwGdOCfLsloZuAqCWBFmRtN claude-statusline release signing'
probe_key='ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAINJQEyz/KfjCqflYdMBC6QbZNIGXSyiyKwMALFSLBlOj probe'
probe_signature='-----BEGIN SSH SIGNATURE-----
U1NIU0lHAAAAAQAAADMAAAALc3NoLWVkMjU1MTkAAAAg0lATLP8p+MKp+Vh0wELpBtk0gZ
dLKLIrAwAsVIsGU6MAAAASc3RhdHVzbGluZS1yZWxlYXNlAAAAAAAAAAZzaGE1MTIAAABT
AAAAC3NzaC1lZDI1NTE5AAAAQGDQeot4d5Dx1rwD4PvikPlszzHbLJLi9U02s28ojvl2eB
6p1Dgr8oaqROcYx988k/36SNWHGORE1c/f7c980ws=
-----END SSH SIGNATURE-----'

tag_language() {
  case "${1%%[_.@-]*}" in
    [Ff][Rr]) echo fr ;;
    [Ee][Nn]) echo en ;;
  esac
}

choose_language() {
  forced=$(tag_language "${STATUSLINE_LANG:-}")
  if [ -n "$forced" ]; then
    echo "$forced"
    return
  fi
  locale=${LC_ALL:-${LC_MESSAGES:-${LANG:-}}}
  case "${locale%%[_.@-]*}" in
    [Cc] | [Pp][Oo][Ss][Ii][Xx])
      echo en
      return
      ;;
  esac
  old_ifs=$IFS
  IFS=:
  for entry in ${LANGUAGE:-}; do
    found=$(tag_language "$entry")
    if [ -n "$found" ]; then
      IFS=$old_ifs
      echo "$found"
      return
    fi
  done
  IFS=$old_ifs
  found=$(tag_language "$locale")
  echo "${found:-en}"
}

language=$(choose_language)

show() {
  case "$language.$1" in
    en.build_failed) text="build failed, using the released binary" ;;
    fr.build_failed) text="compilation impossible, passage au binaire publié" ;;
    en.no_asset) text="no released binary for $2 $3: install Rust (https://rustup.rs) and run this script again from a clone of the repository" ;;
    fr.no_asset) text="aucun binaire publié pour $2 $3 : installez Rust (https://rustup.rs) puis relancez depuis un clone du dépôt" ;;
    en.downloading) text="downloading $2" ;;
    fr.downloading) text="téléchargement de $2" ;;
    en.download_failed) text="download failed: check the network, or install Rust (https://rustup.rs) and run this script again from a clone of the repository" ;;
    fr.download_failed) text="téléchargement impossible : vérifiez le réseau, ou installez Rust (https://rustup.rs) puis relancez depuis un clone du dépôt" ;;
    en.checksum) text="SHA-256 hash missing from the release manifest or different: binary refused" ;;
    fr.checksum) text="empreinte SHA-256 absente du manifeste de publication ou différente : binaire refusé" ;;
    en.signature) text="signature of the release manifest missing or invalid: binary refused" ;;
    fr.signature) text="signature du manifeste de publication absente ou invalide : binaire refusé" ;;
    en.no_signature_check) text="warning: ssh-keygen cannot check signatures here, so the signature of the release was not verified, only its SHA-256 hash" ;;
    fr.no_signature_check) text="attention : ssh-keygen ne peut pas vérifier de signature ici, la signature de la publication n'a pas été vérifiée, seulement son empreinte SHA-256" ;;
    en.copied) text="program copied to $2" ;;
    fr.copied) text="programme copié dans $2" ;;
    en.config_kept) text="statusline.json already exists, kept" ;;
    fr.config_kept) text="statusline.json existe déjà, conservé" ;;
    en.config_installed) text="statusline.json installed" ;;
    fr.config_installed) text="statusline.json installé" ;;
    en.account_kept) text="account left as is: $2" ;;
    fr.account_kept) text="compte laissé tel quel : $2" ;;
    en.account_missing) text="account folder not found, skipped: $2" ;;
    fr.account_missing) text="dossier de compte introuvable, ignoré : $2" ;;
    en.done) text="done, restart Claude Code" ;;
    fr.done) text="terminé, relancez Claude Code" ;;
    en.usage) text="usage: sh install.sh [--uninstall]" ;;
    fr.usage) text="usage : sh install.sh [--uninstall]" ;;
    en.no_program) text="program not found: $2. Nothing was changed, remove the statusLine block of each account by hand." ;;
    fr.no_program) text="programme introuvable : $2. Rien n'a été modifié, retirez à la main le bloc statusLine de chaque compte." ;;
    en.uninstalled) text="statusLine removed from the accounts, the program and statusline.json are still there" ;;
    fr.uninstalled) text="statusLine retirée des comptes, le programme et statusline.json sont toujours là" ;;
  esac
  printf '%s\n' "$text"
}

flag=--install
case "${1:-}" in
  "") ;;
  --uninstall) flag=--uninstall ;;
  *)
    show usage >&2
    exit 2
    ;;
esac

ask_language() {
  asked=$("$bin_dir/statusline" --language 2>/dev/null) || asked=""
  case "$asked" in
    fr | en) language=$asked ;;
  esac
  STATUSLINE_LANG=$language
  export STATUSLINE_LANG
}

seen=""
install_for() {
  d=${1%/}
  [ -d "$d" ] || return 0
  case "|$seen|" in *"|$d|"*) return 0 ;; esac
  seen="$seen|$d"
  CLAUDE_CONFIG_DIR="$d" "$bin_dir/statusline" "$flag" || show account_kept "$d" >&2
}

each_account() {
  install_for "$main"
  for d in "$HOME"/.claude-compte*; do
    install_for "$d"
  done
  reg="$HOME/.claude-accounts.json"
  if [ -f "$reg" ]; then
    grep -o '"Dir"[[:space:]]*:[[:space:]]*"[^"]*"' "$reg" | sed 's/.*"\([^"]*\)"$/\1/' | while IFS= read -r rel; do
      if [ -d "$HOME/$rel" ]; then install_for "$HOME/$rel"; else show account_missing "$HOME/$rel" >&2; fi
    done
  fi
}

if [ "$flag" = --uninstall ]; then
  if [ ! -x "$bin_dir/statusline" ]; then
    show no_program "$bin_dir/statusline" >&2
    exit 1
  fi
  ask_language
  each_account
  show uninstalled
  exit 0
fi

is_repo() {
  [ -f "$here/src/main.rs" ] && [ -f "$here/Cargo.toml" ] || return 1
  tr -d '\r' < "$here/Cargo.toml" | awk '
    substr($0, 1, 1) == "[" { in_package = ($0 == "[package]") }
    in_package && /^name[[:space:]]*=[[:space:]]*"statusline"[[:space:]]*$/ { found = 1 }
    END { exit !found }'
}

is_loopback_release() {
  case "$1" in
    http://127.0.0.1:*/) port=${1#http://127.0.0.1:}; port=${port%/} ;;
    *) return 1 ;;
  esac
  case "$port" in '' | *[!0-9]*) return 1 ;; esac
  [ "${#port}" -le 5 ]
}

check_signature() {
  printf 'statusline-release namespaces="statusline-release" %s\n' "$1" > "$staged.allowed"
  "$keygen" -Y verify -f "$staged.allowed" -I statusline-release -n statusline-release -s "$3" < "$2" > /dev/null 2>&1
}

verify_manifest() {
  keygen=$(command -v ssh-keygen 2>/dev/null) || keygen=""
  printf 'statusline probe\n' > "$staged.probe"
  printf 'statusline probe!\n' > "$staged.tampered"
  printf '%s\n' "$probe_signature" > "$staged.probe.sig"
  if [ -n "$keygen" ] && check_signature "$probe_key" "$staged.probe" "$staged.probe.sig" &&
    ! check_signature "$probe_key" "$staged.tampered" "$staged.probe.sig"; then
    check_signature "$release_key" "$1" "$2"
  else
    show no_signature_check >&2
  fi
}

can_build() { is_repo && command -v cargo >/dev/null 2>&1; }

mkdir -p "$bin_dir"
staged="$bin_dir/statusline.install-$$"
find "$bin_dir" -name 'statusline.install-*' -mmin +10 -exec rm -f {} + 2>/dev/null || true
if can_build && (cd "$here" && cargo build --release --quiet --target-dir "$here/target"); then
  cp "$here/target/release/statusline" "$staged"
else
  can_build && show build_failed >&2
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
  release_base="$repo/releases/latest/download"
  release_protocol=https
  if is_loopback_release "${STATUSLINE_RELEASE_URL:-}"; then
    release_base="${STATUSLINE_RELEASE_URL%/}"
    release_protocol=http
  fi
  fetch() { curl -q -fsSL --proto "=$release_protocol" --proto-redir "=$release_protocol" "$release_base/$1" -o "$2"; }
  sha256() { if command -v sha256sum >/dev/null 2>&1; then sha256sum; else shasum -a 256; fi < "$1" | cut -d ' ' -f 1; }
  clean_download() { rm -f "$staged" "$staged.manifest" "$staged.manifest.sig" "$staged.allowed" "$staged.probe" "$staged.tampered" "$staged.probe.sig"; }
  fetch release-manifest "$staged.manifest" && fetch release-manifest.sig "$staged.manifest.sig" && fetch "$asset" "$staged" || {
    clean_download
    show download_failed >&2
    exit 1
  }
  verify_manifest "$staged.manifest" "$staged.manifest.sig" || {
    clean_download
    show signature >&2
    exit 1
  }
  expected=$(awk -v name="$asset" '
    NR == 1 { ok = ($0 ~ /^tag v[0-9]+\.[0-9]+\.[0-9]+$/); next }
    NF == 2 && ($2 == name || $2 == "*" name) { count++; hash = $1 }
    END { if (ok && count == 1 && length(hash) == 64 && hash !~ /[^0-9a-f]/) print hash }' "$staged.manifest")
  if [ -z "$expected" ] || [ "$(sha256 "$staged")" != "$expected" ]; then
    clean_download
    show checksum >&2
    exit 1
  fi
  rm -f "$staged.manifest" "$staged.manifest.sig" "$staged.allowed" "$staged.probe" "$staged.tampered" "$staged.probe.sig"
fi

chmod 755 "$staged"
mv -f "$staged" "$bin_dir/statusline"
ask_language
show copied "$bin_dir/statusline"

if [ -f "$main/statusline.json" ]; then
  show config_kept
elif is_repo && [ -f "$here/statusline.json" ]; then
  cp "$here/statusline.json" "$main/statusline.json"
  show config_installed
fi

each_account
show done
