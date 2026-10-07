# claude-statusline

Barre d'état pour Claude Code sous Windows : contexte utilisé, quotas 5 h et 7 j avec le temps avant la remise à zéro, modèle et effort, puis RAM, disque et processeur sur une seconde ligne. Les barres sont faites de carrés à dégradé, la couleur passe du violet au rouge pour le quota 5 h et du vert au rouge pour le 7 j selon le remplissage.

Le dépôt contient deux versions au rendu identique. La plus récente est une mod Claude Code (`mod/barre-etat`) qui dessine la barre dans la bande au-dessus du prompt. L'ancienne est `statusline.exe`, branchée par le réglage `statusLine` et affichée sous le prompt.

## Installation

Le guide complet, avec la vérification, la mise à jour et la désinstallation, est dans [INSTALL.md](INSTALL.md). Version courte pour la mod :

```powershell
git clone https://github.com/LetermeFlorent/claude-statusline.git
cd claude-statusline
powershell -ExecutionPolicy Bypass -File .\install-mod.ps1
```

Le script copie la mod dans le dossier `skills` de `.claude` et de chaque `.claude-compte*`, d'où Claude Code la charge à chaque session. Les comptes dont le dossier `skills` est une jonction vers celui du premier compte partagent la même copie. L'ancienne barre reste en place tant qu'on ne lance pas le script avec `-RetirerAncienne`, on voit donc les deux le temps de comparer.

Pour l'ancienne version seule, lancer `install.ps1` à la place.

## Réglages

Les deux versions lisent `%USERPROFILE%\.claude\statusline.json`, ou le fichier désigné par la variable `STATUSLINE_CONFIG`. On y règle la largeur des barres (`bar_width`), les glyphes, les segments affichés (`ctx`, `5h`, `7d`, `model`), la force du dégradé, les couleurs et les métriques système (`ram`, `io`, `cpu`). Pour un terminal sombre, mettre `terminal_background` à `dark`. La mod relit ce fichier toutes les 3 secondes.

## Fonctionnement de la mod

Le contexte, le modèle et l'effort viennent de la session. Les quotas sont lus sur l'API d'Anthropic dès le lancement, puis remplacés par ceux que renvoie chaque réponse. La lecture est gardée deux minutes par compte, ce qui évite de solliciter l'API à chaque nouvelle fenêtre. Quand `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` est défini, Claude Code ne prête pas son jeton aux mods : la mod prend alors celui du compte dans `.credentials.json`, comme le menu multi-comptes.

La ligne système a besoin des compteurs de Windows, auxquels une mod n'a pas accès. Au premier lancement, la mod compile un petit programme C# (`helper/sysinfo.cs`) avec le `csc.exe` livré avec .NET Framework 4, présent sur Windows 10 et 11. Le résultat est rangé dans `%LOCALAPPDATA%\claude-statusline` et recompilé seulement si la source change.

Le `[-]` gris à droite de la bande est dessiné par Claude Code, pas par la mod.

## Tests

Les tests de la mod tournent dans le banc d'essai de Claude Code, sans réseau ni vrai système :

```powershell
cd mod\barre-etat
claude plugin validate .
claude plugin test
```

## Limites

Les deux versions ne tournent que sous Windows x64. La mod a été testée avec Claude Code 2.1.292, avec et sans le rendu plein écran. Le code source Rust de `statusline.exe` n'a pas été retrouvé, seul le binaire est versionné ; la mod reprend son rendu à l'identique et le remplace.
