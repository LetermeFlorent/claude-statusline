# Installer claude-statusline

Ce guide installe la barre d'état dans Claude Code et explique comment la vérifier, la régler, la mettre à jour et la retirer, sous Windows, Linux et macOS.

## Avant de commencer

Il faut Claude Code installé. Sous Windows, le dépôt contient un binaire prêt (`bin/statusline.exe`, Windows 10 ou plus récent en 64 bits). Sous Linux et macOS, `install.sh` compile les sources si Rust est présent (https://rustup.rs) et télécharge sinon le binaire publié pour le système.

Les carrés de la barre sont des caractères Unicode. Les terminaux courants (Windows Terminal, le terminal de VS Code, iTerm2, Terminal.app, GNOME Terminal, Konsole) les affichent sans réglage.

## Installer sous Windows

```powershell
git clone https://github.com/LetermeFlorent/claude-statusline.git
cd claude-statusline
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

Le script copie le programme dans `%USERPROFILE%\.claude\bin\statusline.exe` et installe `statusline.json` dans `%USERPROFILE%\.claude` seulement s'il n'existe pas. Il lance ensuite `statusline.exe --install` pour `.claude`, chaque dossier `.claude-compte*` et chaque compte listé dans `.claude-accounts.json`. Cette commande garde une copie `settings.json.bak-<date>` du fichier de réglages, puis y pose le bloc `statusLine` sans toucher au reste. Un `settings.json` illisible est laissé tel quel, avec un avertissement.

Avec `-Build`, le script compile les sources (Rust requis) au lieu de prendre le binaire du dépôt.

## Installer sous Linux ou macOS

```sh
git clone https://github.com/LetermeFlorent/claude-statusline.git
cd claude-statusline
sh install.sh
```

Le script fait la même chose que sous Windows, avec le programme dans `~/.claude/bin/statusline`. Sans `cargo`, il télécharge le binaire de la dernière version publiée pour Linux x86_64 ou arm64, ou macOS arm64 ou Intel.

## Vérifier

Fermer puis relancer Claude Code : la barre s'affiche sous le prompt. Pour essayer le programme seul, sans Claude Code :

```sh
~/.claude/bin/statusline --demo
```

Sous Windows, `& "$env:USERPROFILE\.claude\bin\statusline.exe" --demo`. Deux lignes de barres doivent s'afficher. Si rien n'apparaît dans Claude Code alors que ce test fonctionne, ouvrir le `settings.json` du compte et vérifier la présence du bloc suivant, avec le chemin du poste :

```json
"statusLine": {
  "command": "C:/Users/<utilisateur>/.claude/bin/statusline.exe",
  "refreshInterval": 3,
  "type": "command"
}
```

## Régler l'apparence

Tout se règle dans `~/.claude/statusline.json`, relu à chaque rafraîchissement. Les couleurs suivent seules le fond du terminal ; `statusline --theme` affiche le fond détecté (`light` ou `dark`). Si la détection se trompe, mettre `terminal_background` à `light` ou `dark` dans le fichier, ou définir la variable `STATUSLINE_BG`.

`bar_width` change le nombre de carrés, `segments` active ou masque `ctx`, `5h`, `7d` et `model`, et `gradient` règle la force du dégradé. Le bloc `system` pilote la seconde ligne : retirer une métrique de la liste `metrics` (`ram`, `io`, `cpu`) la masque, `"enabled": false` supprime la ligne entière et `disk_path` choisit le disque suivi. Pour utiliser un autre fichier, définir `STATUSLINE_CONFIG` avec son chemin.

## Mettre à jour

Faire `git pull` dans le dossier cloné et relancer le script d'installation. Le programme est remplacé, même si une session Claude Code tourne ; `statusline.json` et le reste des réglages sont conservés.

## Désinstaller

`statusline --uninstall` retire le bloc `statusLine` du compte courant après une copie de sauvegarde. Pour un autre compte, définir `CLAUDE_CONFIG_DIR` avec son dossier avant de lancer la commande. Supprimer ensuite `~/.claude/bin/statusline` (`statusline.exe` sous Windows), puis, si on n'en veut plus, `~/.claude/statusline.json` et le dossier d'état décrit dans le README.

## En cas de problème

Si Windows bloque le programme après un téléchargement en ZIP, lancer `Unblock-File "$env:USERPROFILE\.claude\bin\statusline.exe"`. Il n'est pas signé numériquement, ce qui peut déclencher un avertissement de SmartScreen ou de l'antivirus.

Si les quotas 5 h et 7 j affichent `--%`, Claude Code ne les a pas encore transmis pour cette session ; la barre se remplit après la première réponse. Avec claude-account-menu, le hook `SessionStart` les remplit dès le lancement. Le 5 h reste à `--%` tant que le compte n'a pas de fenêtre de 5 h ouverte, c'est-à-dire avant son premier message.

Le disque et le processeur affichent `--%` au premier rafraîchissement, le temps d'avoir deux relevés à comparer.
