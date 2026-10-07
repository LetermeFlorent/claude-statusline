# claude-statusline

Barre d'état pour Claude Code, sous Windows, Linux et macOS. Elle tient sur deux lignes : la première suit la session (contexte, quotas 5 h et 7 j, modèle et effort), la seconde suit le poste (RAM, activité du disque, processeur).

```
ctx ■■▪■■■■■ 340k/1M | 5h ■■▪■■■■■ 36% 3h29m | 7d ■■■■▪■■■ 61% 3d16h | Opus 5:max
ram ■■■■■▪■■ 11.3/15.3G | C: ▪■■■■■■■ 2% | cpu ▪■■■■■■■ 8%
```

C'est un petit programme Rust sans dépendance. Claude Code le lance toutes les 3 secondes par le réglage `statusLine` : il lit le JSON de la session sur son entrée standard, écrit la barre sur sa sortie standard et rend la main en un peu plus d'une milliseconde de travail. Sous Windows, le binaire se passe de la bibliothèque C et pèse environ 200 Ko.

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

Les deux scripts posent la barre dans `.claude`, dans chaque dossier `.claude-compte*` et dans chaque compte listé par le menu multi-comptes ([claude-account-menu](https://github.com/LetermeFlorent/claude-account-menu)). `install.ps1` prend le binaire fourni dans `bin`, ou compile les sources avec `-Build`. `install.sh` compile quand `cargo` est présent, et télécharge le binaire de la dernière version publiée quand `cargo` manque ou que la compilation échoue.

## Ce que montre la barre

La première ligne enchaîne quatre segments séparés par `|`, toujours dans cet ordre :

- `ctx` : remplissage du contexte. Le texte donne les jetons utilisés sur la taille de la fenêtre (`340k/1M`), ou le seul pourcentage quand Claude Code ne transmet pas les deux, ou `--%` sans aucune donnée. La barre va du bleu clair au bleu soutenu.
- `5h` : quota de la fenêtre de 5 heures, en pourcentage, puis le temps avant sa remise à zéro (`3h29m`, ou `45m` sous une heure). Sans heure de remise à zéro connue, il affiche `--` à la place.
- `7d` : quota de la semaine, avec le temps avant remise à zéro en jours et heures au-delà d'un jour (`3d16h`).
- le modèle, souligné, suivi de `:` et du niveau d'effort. Les lettres de l'effort sont en dégradé, avec une teinte propre à `low`, `medium`, `high`, `xhigh` et `max`, et une teinte commune aux autres valeurs. `auto` prend la teinte de `low`, et `ultracode` s'affiche `ultracode(+workflows)`. Le nom du modèle est coupé avant sa première parenthèse : `Opus 4 (1M context)` devient `Opus 4`.

La seconde ligne montre trois métriques du poste :

- `ram` : mémoire utilisée sur mémoire totale, en Gio avec une décimale. La mémoire utilisée est le total moins la mémoire disponible.
- le disque, étiqueté par sa lettre sous Windows (`C:`) et `dsk` ailleurs : part du temps passée en lectures et écritures depuis le rafraîchissement précédent.
- `cpu` : occupation du processeur depuis le rafraîchissement précédent. Sous Linux, l'attente des entrées-sorties ne compte pas comme occupation.

Le disque et le processeur sont des écarts entre deux relevés : ils affichent `--%` au tout premier passage, puis dès que la source ne répond pas. Ils viennent des compteurs du système, `GetSystemTimes` et `IOCTL_DISK_PERFORMANCE` sous Windows, `/proc/stat`, `/proc/diskstats` et `/proc/meminfo` sous Linux, Mach et IOKit sous macOS. Le pourcentage est plafonné à 100.

Les jetons s'écrivent tels quels sous 1000, puis en `k`, `M` et `B` arrondis à l'unité.

### Couleurs des barres

Une barre de quota prend sa teinte selon son niveau : la teinte normale sous 30 %, la teinte d'alerte à partir de 30 %, la teinte critique à partir de 70 %. Pour le 5 h, c'est violet, puis mauve, puis rouge framboise. Pour le 7 j, vert, puis ocre, puis rouge. Les métriques du poste gardent leur teinte propre (bleu-vert pour la RAM, kaki pour le disque, bleu pour le processeur) jusqu'à 70 %, passent à l'orange au-delà, puis au rouge au-delà de 85 %.

Dans une barre, les carrés s'éclaircissent légèrement de gauche à droite sur fond sombre, et foncent sur fond clair. Le dernier carré rempli est plus petit (`▪`) pour marquer la fin. Le remplissage se calcule au demi-carré près, et toute valeur non nulle remplit au moins un carré. Quand la valeur atteint 100 %, le chiffre prend la couleur de la barre au lieu de celle du texte. Étiquettes et chiffres sont en gras.

### Quand un élément disparaît

Les segments `5h` et `7d` restent masqués tant qu'aucun quota n'est connu pour le compte : ni dans le JSON de la session, ni dans le fichier d'état depuis moins de 10 minutes. La seconde ligne disparaît quand `system.enabled` vaut `false`, ou quand la liste `system.metrics` ne contient aucun nom reconnu. Chaque segment peut aussi être masqué par le réglage `segments`. Si les quatre le sont, la première ligne reste vide.

## Quotas 5 h et 7 j

Claude Code transmet les quotas dans le JSON de la session (`rate_limits.five_hour` et `rate_limits.seven_day`, avec `used_percentage` et `resets_at`), à partir de la première réponse de l'API. La barre les affiche et les range dans le fichier d'état sous les clés du compte.

Avant cette première réponse, elle reprend les valeurs gardées pour le même compte si elles ont moins de 10 minutes. Une fenêtre dont l'heure de remise à zéro est passée garde une heure, avancée à l'échéance suivante, mais perd son pourcentage, qui s'affiche `--%`. Le 5 h reste à `--%` tant que le compte n'a pas de fenêtre de 5 heures ouverte, c'est-à-dire avant son premier message.

Le compte est reconnu au nom de son dossier de configuration : `CLAUDE_CONFIG_DIR` s'il est défini et non vide, sinon `~/.claude`. Le nom perd son point initial, passe en minuscules, et tout caractère hors `a-z`, `0-9`, `.`, `_` et `-` devient `_`. `.claude-compte2` donne `claude-compte2`, et un nom vide donne `claude`. Plusieurs comptes ouverts en même temps partagent le fichier d'état sans se mélanger.

Le hook `SessionStart` de [claude-account-menu](https://github.com/LetermeFlorent/claude-account-menu) écrit les mêmes clés au démarrage de chaque session, d'après l'API de suivi d'Anthropic : les quotas s'affichent alors dès l'ouverture.

## Seuil d'auto-compactage

Quand un seuil d'auto-compactage plus petit que la fenêtre de contexte est réglé, la barre `ctx` se mesure par rapport à ce seuil, le texte devient `utilisés/seuil`, et un `ª` discret apparaît devant `ctx`. Le seuil se cherche dans cet ordre :

1. la variable d'environnement `CLAUDE_CODE_AUTO_COMPACT_WINDOW` du processus, si c'est un entier positif ;
2. sinon trois fichiers, lus l'un après l'autre : le `settings.json` du dossier de configuration du compte, puis `.claude/settings.json` et `.claude/settings.local.json` du projet. Le dernier qui donne un seuil l'emporte.

Dans chaque fichier, `env.CLAUDE_CODE_AUTO_COMPACT_WINDOW` (un entier écrit en texte) passe avant `autoCompactWindow` (un nombre entier), et `"autoCompactEnabled": false` annule le seuil trouvé dans un fichier précédent. Le projet est le dossier `cwd` transmis par Claude Code, à défaut le dossier courant du programme.

Les fichiers sont relus au plus toutes les 30 secondes, et seulement si leur taille ou leur date a changé : un nouveau réglage peut mettre 30 secondes à se voir. Le seuil n'est pas cherché quand le segment `ctx` est masqué.

## Thème clair ou sombre

La barre choisit seule ses couleurs selon le fond du terminal. Elle prend la première réponse dans cet ordre :

1. la variable `STATUSLINE_BG` ;
2. le réglage `terminal_background` de `statusline.json` ;
3. la variable `COLORFGBG` posée par certains terminaux : le nombre après le dernier `;` est la couleur du fond, et 7 ou 9 et plus veulent dire fond clair ;
4. les réglages du terminal hôte, s'il est reconnu ;
5. le thème du système.

`STATUSLINE_BG` et `terminal_background` acceptent `light` ou `clair`, `dark` ou `sombre`, sans tenir compte des majuscules. Sans réponse, la barre part sur un fond sombre.

### Terminaux hôtes reconnus

Le terminal est reconnu par ses variables d'environnement, puis ses réglages sont lus :

- VS Code et ses dérivés (`TERM_PROGRAM=vscode`, ou `VSCODE_INJECTION`, ou `VSCODE_PID`). Le fichier est `User/settings.json` de Code, Code - Insiders, VSCodium, Cursor ou Windsurf, dans `%APPDATA%` sous Windows, `~/Library/Application Support` sous macOS, `$XDG_CONFIG_HOME` ou `~/.config` sous Linux. Le produit qui a lancé le terminal, repéré par `VSCODE_GIT_ASKPASS_NODE`, `VSCODE_GIT_ASKPASS_MAIN` ou `VSCODE_CWD`, est lu en premier. `"window.autoDetectColorScheme": true` renvoie au thème du système. Sinon, un `workbench.colorTheme` dont le nom contient `light` donne un fond clair, tout autre nom ou son absence un fond sombre.
- Windows Terminal (`WT_SESSION`). Le fichier est le `settings.json` du paquet Microsoft Store (version stable d'abord, puis Preview et Canary), ou celui de l'installation hors Store. Le profil retenu est celui de `WT_PROFILE_ID`, sinon le profil par défaut. Sa couleur `background`, ou celle de `profiles.defaults`, décide par sa luminance. À défaut, son jeu de couleurs décide : un jeu à deux variantes `{ "light", "dark" }` suit la clé `theme` du fichier, puis le système ; un jeu personnalisé de `schemes` décide par la luminance de son fond ; parmi les jeux fournis, `One Half Light`, `Solarized Light` et `Tango Light` sont clairs, les autres sombres. Sans jeu indiqué, c'est `Campbell`, sombre.
- Zed (`TERM_PROGRAM=zed`), dans `%APPDATA%\Zed\settings.json` sous Windows et `~/.config/zed/settings.json` ailleurs. Un `theme` texte dont le nom contient `light` donne un fond clair. Un `theme` objet suit son `mode` (`light`, `dark`), et `system` ou l'absence de réglage renvoient au système.
- Ghostty (`TERM_PROGRAM=ghostty`), dans `~/.config/ghostty/config` et, sous macOS, `~/Library/Application Support/com.mitchellh.ghostty/config`, lu en dernier. Une couleur `background` décide par sa luminance. Sinon `theme` : une valeur `light:...,dark:...` suit le système, un nom qui contient `light` est clair, le reste sombre.

Les autres terminaux passent directement au thème du système.

### Thème du système

Chaque système a sa propre source :

- Windows : la valeur `AppsUseLightTheme` du registre de l'utilisateur (applications en mode clair). Une valeur jamais posée veut dire mode clair, comme dans Windows.
- macOS : la clé `AppleInterfaceStyle` de `~/Library/Preferences/.GlobalPreferences.plist`. `Dark` donne un fond sombre, son absence un fond clair.
- Linux : `GTK_THEME` (sombre s'il contient `dark`), puis sous KDE la couleur `BackgroundNormal` de la section `[Colors:Window]` de `kdeglobals`, puis `gsettings` : `color-scheme` (`prefer-dark` ou `prefer-light`), et à défaut le nom de `gtk-theme`.

Quand le terminal suit le système (VS Code avec `window.autoDetectColorScheme`, un jeu de couleurs Windows Terminal à deux variantes, Zed en `system`), c'est donc le thème du système qui tranche.

Le résultat des étapes 4 et 5 est gardé dans le fichier d'état. Il est recalculé dès qu'un fichier de réglages du terminal change de taille ou de date, ou que le thème du système change : un passage en mode clair se voit au rafraîchissement suivant. Les étapes 1 à 3 sont relues à chaque passage. `statusline --theme` affiche le fond retenu.

## Réglages

Le programme cherche son fichier de réglages dans cet ordre :

1. le fichier désigné par `STATUSLINE_CONFIG`, même s'il n'existe pas ;
2. `statusline.json` dans le dossier `CLAUDE_CONFIG_DIR`, s'il existe ;
3. `~/.claude/statusline.json`.

Il est relu à chaque rafraîchissement. Le JSON y tolère les commentaires `//` et `/* */`, les virgules en trop et un BOM. Un fichier absent ou illisible laisse toutes les valeurs par défaut, sans message. Les couleurs s'écrivent `#RRGGBB` ou `#RGB`, le `#` est facultatif, et une couleur invalide reprend sa valeur par défaut. Le `statusline.json` du dépôt reprend une partie des valeurs par défaut, et toutes les clés du tableau peuvent y être ajoutées.

| Clé | Par défaut | Effet |
|---|---|---|
| `terminal_background` | absente | `light`, `clair`, `dark` ou `sombre` force le fond |
| `bar_width` | `8` | nombre de carrés par barre, de 1 à 40 |
| `bold` | `true` | `false` retire le gras des étiquettes et des chiffres |
| `glyphs.filled`, `glyphs.empty`, `glyphs.half` | `■`, `■`, `▪` | carré plein, carré vide, dernier carré rempli |
| `auto_compact_marker` | `ª` | marqueur du seuil d'auto-compactage |
| `effort_style` | `gradient` | `flat` colore l'effort d'une seule teinte |
| `segments.ctx`, `segments.5h`, `segments.7d`, `segments.model` | `true` | `false` masque le segment |
| `gradient.5h`, `gradient.7d` | `0.55`, `0.3` | force de l'éclaircissement dans la barre, de 0 à 1 |
| `thresholds.quota_warn`, `thresholds.quota_hot` | `30`, `70` | pourcentages à partir desquels un quota passe en alerte, puis en critique |
| `thresholds.system_warn`, `thresholds.system_hot` | `70`, `85` | pourcentages au-delà desquels une métrique du poste passe en alerte, puis en critique |
| `palettes.5h.ok`, `.warn`, `.hot` | `#786EC8`, `#AA5AAF`, `#C83C5F` | teintes du quota 5 h |
| `palettes.7d.ok`, `.warn`, `.hot` | `#538977`, `#B88143`, `#B95555` | teintes du quota 7 j |
| `colors.ink_light`, `colors.ink_dark` | `#000000`, `#E8EAED` | texte, sur fond clair et sur fond sombre |
| `colors.empty_light`, `colors.empty_dark` | `#BEB4A5`, `#E2E5EA` | carrés vides |
| `colors.ctx_from_light`, `colors.ctx_to_light` | `#7DB9FF`, `#285AB4` | dégradé du contexte sur fond clair |
| `colors.ctx_from_dark`, `colors.ctx_to_dark` | `#96CDFF`, `#4682DC` | dégradé du contexte sur fond sombre |
| `system.enabled` | `true` | `false` supprime la seconde ligne |
| `system.metrics` | `["ram", "io", "cpu"]` | métriques affichées, dans l'ordre de la liste |
| `system.gradient` | `0.45` | force de l'éclaircissement pour la seconde ligne |
| `system.disk_path` | `C:\` sous Windows, `/` ailleurs | disque suivi |
| `system.colors.ram.ok`, `.dsk.ok`, `.cpu.ok` | `#1E787D`, `#697855`, `#375A8C` | teinte normale de chaque métrique |
| `system.colors.<métrique>.warn`, `.hot` | `#BE823C`, `#C83C3C` | teintes d'alerte et critique |

Les noms de `system.metrics` s'écrivent en minuscules, et les noms inconnus sont ignorés. Une liste vide, ou absente, revient aux trois métriques. La couleur du disque se règle sous `dsk`, même si la métrique s'appelle `io`.

`system.disk_path` ne se lit pas de la même façon partout. Sous Windows, seule la première lettre compte : `D:\Jeux` suit le lecteur `D:`. Sous Linux, c'est le disque qui porte ce chemin, et à défaut le premier disque physique de `/proc/diskstats`. Sous macOS, le réglage est ignoré : la barre additionne l'activité de tous les disques.

## Ligne de commande

Le programme s'installe dans `~/.claude/bin`, sous le nom `statusline.exe` sous Windows et `statusline` ailleurs. Ce dossier n'est pas dans le PATH, il faut donc l'appeler par son chemin complet, par exemple `~/.claude/bin/statusline --theme`.

| Commande | Effet |
|---|---|
| sans argument | lit le JSON de Claude Code sur l'entrée standard, écrit la barre et met à jour le fichier d'état. Lancé à la main dans un terminal, il n'attend rien et dessine une barre sans données. |
| `--demo` | rendu d'exemple sans Claude Code (Opus 5, effort `max`, contexte 340k sur 1M, 5 h à 36 %, 7 j à 61 %), avec les vrais réglages, le vrai thème et les vraies métriques du poste. Le fichier d'état n'est pas modifié. |
| `--bench [n]` | rend `n` fois l'exemple (200 par défaut) et affiche le temps moyen par rendu, puis son détail : état, config, session, rendu, écriture. Le lancement du processus n'est pas compté. |
| `--theme` | affiche le fond retenu, `light` ou `dark` |
| `--install` | pose le bloc `statusLine` dans le `settings.json` du compte courant (voir [INSTALL.md](INSTALL.md)) |
| `--uninstall` | retire ce bloc |
| `--version`, `-V` | affiche `statusline` suivi du numéro de version |
| `--help`, `-h` | affiche l'aide |

Seul le premier argument est lu. Le code de sortie vaut 0 en cas de succès, 1 quand `--install` ou `--uninstall` échoue, 2 pour un argument inconnu, qui est rappelé avec l'aide.

## Fichier d'état

Les compteurs, les quotas et le thème détecté sont rangés dans un seul fichier, partagé par tous les comptes :

- Windows : `%LOCALAPPDATA%\claude-statusline\state` ;
- macOS : `~/Library/Application Support/claude-statusline/state` ;
- Linux : `$XDG_STATE_HOME/claude-statusline/state`, à défaut `~/.local/state/claude-statusline/state`.

C'est un fichier texte, une ligne `clé valeur` par entrée. On y trouve les quotas de chaque compte (`q_at@<compte>`, `q5_at@<compte>`, `q5_pct@<compte>`, `q7_at@<compte>`, `q7_pct@<compte>`), le thème (`bg_host`, `bg_stamp`), le seuil d'auto-compactage (`compact_window`, `compact_key`, `compact_stamp`, `compact_at`) et les derniers relevés du disque et du processeur (`io_clock`, `io_value`, `cpu_clock`, `cpu_value`). Les clés inconnues sont conservées. Les clés de la version 1 (`bg_at`, et les quotas sans suffixe de compte) sont retirées au premier passage.

Le fichier n'est réécrit que si son contenu change, par un fichier temporaire ensuite renommé, de sorte qu'aucune lecture ne tombe sur un fichier à moitié écrit. Sans dossier personnel connu (`HOME`, ou `USERPROFILE` sous Windows), rien n'est écrit. `--bench` écrit à côté un fichier `state.bench` et le supprime à la fin.

## Variables d'environnement

Le programme lit les variables suivantes, toutes facultatives :

| Variable | Rôle |
|---|---|
| `STATUSLINE_BG` | force le fond : `light`, `clair`, `dark` ou `sombre` |
| `STATUSLINE_CONFIG` | chemin du fichier de réglages |
| `CLAUDE_CONFIG_DIR` | dossier du compte : réglages, seuil d'auto-compactage, clés de quotas, cible de `--install` |
| `CLAUDE_CODE_AUTO_COMPACT_WINDOW` | seuil d'auto-compactage |
| `COLORFGBG` | couleurs du terminal, pour le fond |
| `TERM_PROGRAM`, `VSCODE_INJECTION`, `VSCODE_PID`, `WT_SESSION` | reconnaissance du terminal hôte |
| `VSCODE_GIT_ASKPASS_NODE`, `VSCODE_GIT_ASKPASS_MAIN`, `VSCODE_CWD` | choix entre VS Code et ses dérivés |
| `WT_PROFILE_ID` | profil Windows Terminal en cours |
| `GTK_THEME`, `XDG_CURRENT_DESKTOP` | thème du système sous Linux |
| `HOME`, `USERPROFILE`, `LOCALAPPDATA`, `APPDATA`, `XDG_STATE_HOME`, `XDG_CONFIG_HOME` | emplacement des fichiers |

Une variable vide compte comme absente.

## Compiler

```sh
cargo build --release
```

Le binaire sort dans `target/release`. Sous Windows, `.cargo/config.toml` lie le programme avec `rust-lld` sans la bibliothèque C, et rien d'autre que Rust n'est nécessaire. Seule la cible `x86_64-pc-windows-msvc` y est prévue. Le profil de publication active `lto`, `panic = "abort"` et retire les symboles.

Les tests (`cargo test`) tournent tels quels sous Linux et macOS. Sous Windows, ils ont besoin de la bibliothèque C de Visual Studio et d'une variable `RUSTFLAGS` qui remplace les options de liaison du fichier de configuration, comme le fait la CI (`RUSTFLAGS=-Cdebuginfo=0`).

## CI et versions publiées

La CI tourne à chaque envoi sur `main`, à chaque étiquette `v*` et sur chaque pull request. Pour chacune des cinq cibles, elle compile en mode publication puis essaie le binaire : version, thème, rendu d'exemple, lecture d'un JSON de session, mesure, installation puis désinstallation, et refus d'un argument inconnu. Elle lance aussi les tests unitaires (sauf pour macOS Intel), `clippy` sous Linux, et un essai complet de `install.sh` sous Linux et macOS.

Une étiquette `v*` publie une version avec ces binaires :

| Fichier | Système |
|---|---|
| `statusline-linux-x86_64` | Linux x86_64, binaire statique (musl) |
| `statusline-linux-arm64` | Linux arm64, binaire statique (musl) |
| `statusline-macos-arm64` | macOS sur puce Apple |
| `statusline-macos-x86_64` | macOS Intel |
| `statusline-windows-x86_64.exe` | Windows 64 bits |

`install.sh` télécharge le fichier de son système dans la dernière version publiée. Sous Windows, `install.ps1` utilise le binaire `bin/statusline.exe` du dépôt, qui est le même programme.

## Limites

Le binaire macOS Intel est compilé par la CI sans y être lancé. Il n'y a pas de binaire Windows ARM64 natif. Sur les autres Unix (BSD par exemple), la barre fonctionne sans métriques du poste ni thème du système : `--%` partout sur la seconde ligne et fond sombre, sauf réglage.

Les couleurs sont envoyées en 24 bits, sans repli sur 256 couleurs, et la variable `NO_COLOR` n'est pas prise en compte. La détection du fond par le terminal hôte couvre VS Code et ses dérivés, Windows Terminal, Zed et Ghostty. Les autres terminaux passent par `COLORFGBG` ou le thème du système, et `STATUSLINE_BG` permet de forcer le choix.
