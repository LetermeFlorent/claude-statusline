# Installer claude-statusline

Ce guide installe la barre d'état dans Claude Code sur un poste Windows et explique comment la vérifier, la régler et la retirer.

## Avant de commencer

Il faut Windows 10 ou plus récent en 64 bits, car le programme est un exécutable Windows x64, et Claude Code installé. Git sert à récupérer le dépôt, mais on peut télécharger l'archive ZIP depuis la page GitHub une fois connecté (le dépôt est privé).

Les carrés de la barre sont des caractères Unicode. Un terminal moderne (Windows Terminal, le terminal intégré de VS Code) les affiche sans réglage ; une très vieille console Windows peut les remplacer par des points d'interrogation.

## Installation

```powershell
git clone https://github.com/LetermeFlorent/claude-statusline.git
cd claude-statusline
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

L'installeur fait quatre choses, dans cet ordre. Il copie `statusline.exe` dans `%USERPROFILE%\.claude\bin`. Il installe `statusline.json` dans `%USERPROFILE%\.claude` seulement si ce fichier n'existe pas, pour ne pas écraser des réglages personnels. Il garde une copie `settings.json.bak-statusline` de chaque fichier de réglages avant de le modifier. Enfin il déclare la barre dans le `settings.json` de `.claude` et de chaque dossier `.claude-compte*` trouvé, ce qui couvre tous les comptes d'un poste multi-comptes.

Fermer puis relancer Claude Code : la barre apparaît en bas de l'écran dès le premier message.

## Vérifier

Pour tester le programme seul, sans Claude Code, envoyer-lui des données d'exemple :

```powershell
'{"context_window":{"used_percentage":30},"rate_limits":{"five_hour":{"used_percentage":36,"resets_at":1893456000}}}' | & "$env:USERPROFILE\.claude\bin\statusline.exe"
```

Une ligne avec des barres de carrés doit s'afficher. Si rien ne s'affiche dans Claude Code alors que ce test fonctionne, ouvrir `%USERPROFILE%\.claude\settings.json` et vérifier la présence du bloc suivant, avec le chemin de votre poste :

```json
"statusLine": {
  "type": "command",
  "command": "C:/Users/<utilisateur>/.claude/bin/statusline.exe",
  "refreshInterval": 3
}
```

## Régler l'apparence

Tout se règle dans `%USERPROFILE%\.claude\statusline.json`, relu à chaque rafraîchissement. Pour un terminal à fond sombre, mettre `terminal_background` à `dark`. `bar_width` change le nombre de carrés, `segments` active ou masque `ctx`, `5h`, `7d` et `model`, et `gradient` règle la force du dégradé. Le bloc `system` pilote la seconde ligne : retirer une métrique de la liste `metrics` (`ram`, `io`, `cpu`) la masque, et `"enabled": false` supprime la ligne entière. Pour utiliser un autre fichier de réglages, définir la variable d'environnement `STATUSLINE_CONFIG` avec son chemin.

## Mettre à jour

Faire `git pull` dans le dossier cloné et relancer `install.ps1`. Le programme est remplacé, les réglages de `statusline.json` sont conservés.

## Désinstaller

Retirer le bloc `statusLine` du `settings.json` de chaque compte, ou y remettre la copie `settings.json.bak-statusline` si aucun autre réglage n'a changé depuis. Supprimer ensuite `%USERPROFILE%\.claude\bin\statusline.exe` et, si on n'en veut plus, `%USERPROFILE%\.claude\statusline.json`.

## En cas de problème

Si Windows bloque l'exécutable après un téléchargement en ZIP, ouvrir ses propriétés et cocher "Débloquer", ou lancer `Unblock-File "$env:USERPROFILE\.claude\bin\statusline.exe"`. L'exécutable n'est pas signé numériquement, ce qui peut déclencher un avertissement de SmartScreen ou de l'antivirus.

Si les quotas 5 h et 7 j affichent `--%`, Claude Code ne les a pas transmis pour cette session ; la barre se remplit quand ils sont disponibles.

Si le script d'installation s'arrête sur une erreur de lecture de `settings.json`, le fichier contient probablement du JSON invalide (une virgule en trop, par exemple). Le corriger, puis relancer l'installeur.
