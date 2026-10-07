#!/bin/sh
# Installe la barre pour tous les comptes Claude Code du poste (Linux, macOS).
# Compile les sources si cargo est present, sinon telecharge le binaire de la derniere version publiee.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
main="$HOME/.claude"
bin_dir="$main/bin"
repo="https://github.com/LetermeFlorent/claude-statusline"

mkdir -p "$bin_dir"
if command -v cargo >/dev/null 2>&1; then
  (cd "$here" && cargo build --release --quiet --target-dir "$here/target")
  cp "$here/target/release/statusline" "$bin_dir/statusline.new"
else
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
  curl -fsSL "$repo/releases/latest/download/$asset" -o "$bin_dir/statusline.new"
fi
# Copie puis renommage : une barre en cours d'execution garde l'ancien fichier jusqu'a la fin
chmod 755 "$bin_dir/statusline.new"
mv -f "$bin_dir/statusline.new" "$bin_dir/statusline"
echo "programme copie dans $bin_dir/statusline"

if [ -f "$main/statusline.json" ]; then
  echo "statusline.json existe deja, conserve"
else
  cp "$here/statusline.json" "$main/statusline.json"
  echo "statusline.json installe"
fi

# Comptes : .claude, chaque .claude-compte*, et ceux declares dans le menu multi-comptes
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
