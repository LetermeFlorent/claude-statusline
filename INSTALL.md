# Installer claude-statusline

Ce guide installe la barre d'état dans Claude Code et explique comment la vérifier, la régler, la mettre à jour et la retirer, sous Windows, Linux et macOS. Le détail de l'affichage et de chaque réglage est dans le [README](README.md).

## Avant de commencer

Il faut Claude Code installé. Le reste dépend du système :

- Windows : rien d'autre. Le dépôt contient un binaire prêt, `bin/statusline.exe`, pour Windows 10 ou plus récent en 64 bits (x86_64). Rust n'est utile que pour compiler soi-même avec `-Build`.
- Linux et macOS : `install.sh` compile les sources si Rust est présent (https://rustup.rs). Sans Rust, il télécharge le binaire publié avec `curl`, pour Linux x86_64 ou arm64 et macOS sur puce Apple ou Intel.

La barre envoie ses couleurs en 24 bits et dessine ses carrés avec des caractères Unicode (`■`, `▪`). Windows Terminal, le terminal de VS Code, iTerm2, GNOME Terminal, Konsole, Ghostty et Zed les affichent sans réglage. Un terminal limité à 256 couleurs montre des teintes approchées ou fausses.

## Installer sous Windows

```powershell
git clone https://github.com/LetermeFlorent/claude-statusline.git
cd claude-statusline
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

Le script fait quatre choses :

1. Il copie le programme dans `%USERPROFILE%\.claude\bin\statusline.exe`. Un ancien programme en cours d'utilisation est d'abord renommé en `.old`, ce qui permet le remplacement même pendant une session Claude Code, puis supprimé quand c'est possible. La copie est débloquée (`Unblock-File`).
2. Il installe `statusline.json` dans `%USERPROFILE%\.claude`, seulement s'il n'existe pas. Les autres comptes lisent ce même fichier, sauf s'ils ont le leur dans leur dossier.
3. Il dresse la liste des comptes : `.claude`, chaque dossier `.claude-compte*` du dossier personnel, et chaque `Dir` de `.claude-accounts.json` (le registre de [claude-account-menu](https://github.com/LetermeFlorent/claude-account-menu)), sans doublon et sans les dossiers absents. Un registre illisible donne un avertissement.
4. Pour chaque compte, il lance `statusline.exe --install` avec `CLAUDE_CONFIG_DIR` pointé sur son dossier. Un échec est signalé par "compte laisse tel quel" et n'arrête pas les autres comptes.

Avec `-Build`, le script compile les sources (`cargo build --release`, Rust requis) au lieu de prendre le binaire du dépôt.

## Installer sous Linux ou macOS

```sh
git clone https://github.com/LetermeFlorent/claude-statusline.git
cd claude-statusline
sh install.sh
```

Le programme va dans `~/.claude/bin/statusline`. Si `cargo` est présent, le script compile les sources dans le dossier `target` du dépôt. Si `cargo` manque, ou si la compilation échoue (le script affiche alors "compilation impossible, passage au binaire publie"), il télécharge le binaire de la dernière version publiée qui correspond à `uname`. Sur un autre système ou une autre architecture, il s'arrête avec un message qui renvoie vers Rust.

Le binaire est d'abord écrit en `statusline.new`, puis renommé, pour qu'une barre en cours d'exécution garde l'ancien fichier jusqu'à la fin. La suite est la même que sous Windows : `statusline.json` installé s'il manque, puis `--install` pour `.claude`, chaque `.claude-compte*` et chaque `Dir` du registre. Le registre est lu par `grep` et `sed`, sans analyseur JSON : il doit garder le format écrit par claude-account-menu.

## Ce que fait --install

`--install` modifie le `settings.json` du compte courant : celui du dossier `CLAUDE_CONFIG_DIR` s'il est défini, sinon `~/.claude/settings.json`. Les étapes sont les suivantes :

- Une copie du fichier est gardée à côté, sous le nom `settings.json.bak-<horodatage>`, où l'horodatage est en secondes depuis 1970 (`-1`, `-2` sont ajoutés en cas de collision). Il n'y a pas de copie si le fichier n'existait pas.
- Le bloc `statusLine` reçoit `"type": "command"` et `"command"` avec le chemin du programme qui exécute la commande, écrit avec des `/`. `refreshInterval` est mis à 3 seulement s'il manque, et les autres clés du bloc sont conservées.
- Le fichier est réécrit avec une indentation de 2 espaces et ses clés triées par ordre alphabétique. Les valeurs ne changent pas, mais la mise en forme et l'ordre des clés, oui.
- Si aucun `statusline.json` n'est trouvé, un modèle réduit est écrit dans le dossier du compte.

Un `settings.json` illisible (JSON invalide, commentaires, fichier vide) n'est jamais touché : la commande affiche "reglages illisibles, rien n'est modifie" et sort avec le code 1. Chaque nouvelle installation laisse une copie `.bak-` de plus, sans jamais purger les anciennes.

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

`--version` affiche la version installée, `--theme` le fond détecté.

## Régler l'apparence

Tout se règle dans `~/.claude/statusline.json`, relu à chaque rafraîchissement, sans relancer Claude Code. Les couleurs suivent seules le fond du terminal, et `statusline --theme` affiche le fond détecté (`light` ou `dark`). Si la détection se trompe, mettre `terminal_background` à `light` ou `dark` dans le fichier, ou définir la variable `STATUSLINE_BG`.

Les réglages courants sont les suivants :

- `bar_width` change le nombre de carrés (1 à 40) ;
- `segments` active ou masque `ctx`, `5h`, `7d` et `model` ;
- `gradient` règle la force de l'éclaircissement des barres 5 h et 7 j, `system.gradient` celle de la seconde ligne ;
- `thresholds` et `palettes` changent les seuils et les teintes des quotas ;
- dans le bloc `system`, `metrics` choisit les métriques et leur ordre (`ram`, `io`, `cpu`), `"enabled": false` supprime la seconde ligne, et `disk_path` choisit le disque suivi sous Windows et Linux.

Une liste `metrics` vide revient aux trois métriques : pour tout masquer, il faut `"enabled": false`. Pour utiliser un autre fichier de réglages, définir `STATUSLINE_CONFIG` avec son chemin. La liste complète des clés, avec leurs valeurs par défaut, est dans le README.

## Mettre à jour

Faire `git pull` dans le dossier cloné et relancer le script d'installation. Le programme est remplacé, même si une session Claude Code tourne. `statusline.json` et le reste des réglages sont conservés, et chaque `settings.json` reçoit une nouvelle copie `.bak-`.

## Désinstaller

`statusline --uninstall` retire le bloc `statusLine` du compte courant, après une copie `settings.json.bak-<horodatage>`, et réécrit le fichier avec ses clés triées. Pour un autre compte, définir `CLAUDE_CONFIG_DIR` avec son dossier avant de lancer la commande. Si le compte n'a pas de bloc `statusLine`, la commande l'indique et ne touche à rien.

Supprimer ensuite le programme, `~/.claude/bin/statusline` (`statusline.exe` sous Windows). Si on n'en veut plus, supprimer aussi `~/.claude/statusline.json`, les éventuels `statusline.json` des dossiers de comptes, les copies `settings.json.bak-*`, et le dossier d'état :

- Windows : `%LOCALAPPDATA%\claude-statusline` ;
- macOS : `~/Library/Application Support/claude-statusline` ;
- Linux : `~/.local/state/claude-statusline`, ou `$XDG_STATE_HOME/claude-statusline`.

## En cas de problème

Si Windows bloque le programme, lancer `Unblock-File "$env:USERPROFILE\.claude\bin\statusline.exe"`. L'installeur le fait déjà, mais le déblocage peut échouer, ou le dépôt peut avoir été récupéré en ZIP. Le programme n'est pas signé numériquement, ce qui peut déclencher un avertissement de SmartScreen ou de l'antivirus.

Si les segments `5h` et `7d` n'apparaissent pas, Claude Code n'a pas encore transmis les quotas pour cette session, et la barre n'a pas de valeur de moins de 10 minutes pour ce compte : ils apparaissent après la première réponse. Avec claude-account-menu, le hook `SessionStart` les remplit dès le lancement. Si le 5 h affiche `--%`, le compte n'a pas de fenêtre de 5 heures ouverte, c'est-à-dire pas encore de message depuis la dernière remise à zéro.

Le disque et le processeur affichent `--%` au premier rafraîchissement, le temps d'avoir deux relevés à comparer. Ils restent à `--%` sur un système autre que Windows, Linux et macOS.

Si les couleurs sont fausses ou les carrés remplacés par des points d'interrogation, le terminal ne gère pas les couleurs 24 bits ou la police n'a pas les caractères `■` et `▪`. Ces caractères se changent dans `glyphs`.

Un changement de seuil d'auto-compactage peut mettre 30 secondes à se voir dans la barre.
