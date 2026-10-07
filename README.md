# claude-statusline

Barre d'état pour Claude Code sous Windows : contexte utilisé, quotas 5 h et 7 j avec le temps avant la remise à zéro, modèle et effort, puis RAM, disque et processeur sur une seconde ligne. Les barres sont faites de carrés à dégradé, la couleur passe du violet au rouge pour le quota 5 h et du vert au rouge pour le 7 j selon le remplissage.

## Installation

```powershell
git clone https://github.com/LetermeFlorent/claude-statusline.git
cd claude-statusline
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

Le script copie `statusline.exe` dans `%USERPROFILE%\.claude\bin`, installe `statusline.json` s'il n'existe pas, puis déclare la barre dans le `settings.json` de `.claude` et de chaque dossier `.claude-compte*`. Une copie `settings.json.bak-statusline` est faite avant chaque modification, et il suffit de relancer Claude Code ensuite.

## Réglages

Le programme lit `%USERPROFILE%\.claude\statusline.json`, ou le fichier désigné par la variable `STATUSLINE_CONFIG`. On y règle la largeur des barres (`bar_width`), les glyphes, les segments affichés (`ctx`, `5h`, `7d`, `model`), la force du dégradé, les couleurs et les métriques système (`ram`, `io`, `cpu`). Pour un terminal sombre, mettre `terminal_background` à `dark`.

## Limites

Seul le binaire compilé est versionné ici, le code source Rust n'a pas été retrouvé sur ce poste. Il ne tourne que sous Windows x64.
