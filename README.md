# claude-statusline

Barre d'état pour Claude Code, sous Windows, Linux et macOS. La première ligne montre le contexte utilisé, les quotas 5 h et 7 j avec le temps avant leur remise à zéro, le modèle et l'effort. La seconde montre la RAM, l'activité du disque et le processeur. Les barres sont faites de carrés en dégradé : du violet au rouge pour le quota 5 h, du vert au rouge pour le 7 j, selon le remplissage.

C'est un petit programme Rust sans dépendance. Claude Code le lance toutes les 3 secondes par le réglage `statusLine`, il lit le JSON de la session sur son entrée standard et rend la barre en un peu plus d'une milliseconde de travail. Sous Windows, le binaire se passe de la bibliothèque C et pèse environ 200 Ko.

## Installation

Le guide complet, avec la vérification, la mise à jour et la désinstallation, est dans [INSTALL.md](INSTALL.md).

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

Les deux scripts posent la barre dans `.claude` et dans chaque compte `.claude-compte*`, plus ceux listés par le menu multi-comptes ([claude-account-menu](https://github.com/LetermeFlorent/claude-account-menu)). `install.sh` compile les sources quand `cargo` est présent et télécharge sinon le binaire de la dernière version publiée.

## Thème clair ou sombre

La barre choisit seule ses couleurs selon le fond du terminal. Elle prend la première réponse dans cet ordre :

1. la variable `STATUSLINE_BG` (`light` ou `dark`) ;
2. le réglage `terminal_background` de `statusline.json` ;
3. la variable `COLORFGBG` posée par certains terminaux ;
4. les réglages du terminal hôte : VS Code et ses dérivés (Cursor, VSCodium, Windsurf), Windows Terminal, Zed, Ghostty ;
5. le thème du système : Windows (applications en mode clair), macOS (apparence sombre), Linux (GTK, KDE, GNOME).

Sans réponse, la barre part sur un fond sombre. Quand le terminal suit le système (VS Code avec `window.autoDetectColorScheme`, un jeu de couleurs Windows Terminal à deux variantes), c'est le thème du système qui tranche. Le résultat est gardé dans le fichier d'état et recalculé dès qu'un des fichiers de réglages ou le thème du système change, donc un passage en mode clair se voit au rafraîchissement suivant. `statusline --theme` affiche ce que la barre a détecté.

## Réglages

Le programme lit `~/.claude/statusline.json`, ou le fichier désigné par `STATUSLINE_CONFIG`. Un compte lancé avec `CLAUDE_CONFIG_DIR` peut avoir le sien dans son dossier. On y règle la largeur des barres (`bar_width`), les glyphes, les segments affichés (`ctx`, `5h`, `7d`, `model`), la force des dégradés, les couleurs et les métriques système (`ram`, `io`, `cpu`). `system.disk_path` choisit le disque suivi ; par défaut c'est `C:\` sous Windows et la racine ailleurs. Le fichier fourni dans le dépôt reprend les valeurs par défaut.

## Fonctionnement

Le contexte, le modèle et l'effort viennent du JSON de la session. Les quotas aussi, dès que Claude Code les transmet, c'est-à-dire après la première réponse de l'API. Avant, la barre reprend les dernières valeurs connues du même compte si elles ont moins de 10 minutes. Le hook `SessionStart` de claude-account-menu les rafraîchit à chaque lancement, ce qui remplit la barre dès l'ouverture.

Quand un seuil d'auto-compactage est réglé (`autoCompactWindow` ou `CLAUDE_CODE_AUTO_COMPACT_WINDOW`, dans l'environnement ou dans les réglages du compte et du projet), la barre du contexte se mesure par rapport à ce seuil et un `ª` discret le signale devant `ctx`.

Les pourcentages du disque et du processeur sont des écarts entre deux rafraîchissements : ils restent à `--%` au tout premier affichage. Ils viennent des compteurs du système (`GetSystemTimes` et `IOCTL_DISK_PERFORMANCE` sous Windows, `/proc` sous Linux, Mach et IOKit sous macOS). L'état est rangé dans `%LOCALAPPDATA%\claude-statusline\state` sous Windows, `~/Library/Application Support/claude-statusline/state` sous macOS et `$XDG_STATE_HOME/claude-statusline/state` (à défaut `~/.local/state`) sous Linux.

`statusline --demo` affiche un rendu d'exemple sans Claude Code, `statusline --bench` mesure le coût du travail seul.

## Compiler

```sh
cargo build --release
```

Le binaire sort dans `target/release`. Sous Windows, `.cargo/config.toml` lie le programme avec `rust-lld` sans la bibliothèque C, rien d'autre que Rust n'est nécessaire. Les tests (`cargo test`) tournent tels quels sous Linux et macOS. Sous Windows, ils ont besoin de la bibliothèque C de Visual Studio et d'une variable `RUSTFLAGS` qui remplace les options de liaison du fichier de configuration, comme le fait la CI.

La CI compile et essaie le programme sur les trois systèmes à chaque envoi sur `main`. Une étiquette `v*` publie les binaires pour Linux (x86_64, arm64), macOS (arm64, Intel) et Windows.

## Limites

Le binaire macOS Intel est compilé par la CI sans y être lancé. La détection du fond par le terminal hôte couvre VS Code et ses dérivés, Windows Terminal, Zed et Ghostty ; les autres terminaux passent par `COLORFGBG` ou le thème du système, et `STATUSLINE_BG` permet de forcer le choix.
