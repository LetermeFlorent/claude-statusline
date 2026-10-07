# Installer claude-statusline

Ce guide installe la barre d'état dans Claude Code sur un poste Windows et explique comment la vérifier, la régler et la retirer. La mod est décrite en premier, l'ancien `statusline.exe` ensuite.

## Avant de commencer

Il faut Windows 10 ou plus récent en 64 bits et Claude Code installé. La mod a été testée avec Claude Code 2.1.292. Git sert à récupérer le dépôt, mais on peut télécharger l'archive ZIP depuis la page GitHub une fois connecté (le dépôt est privé).

Les carrés de la barre sont des caractères Unicode. Un terminal moderne (Windows Terminal, le terminal intégré de VS Code) les affiche sans réglage ; une très vieille console Windows peut les remplacer par des points d'interrogation.

## Installer la mod

```powershell
git clone https://github.com/LetermeFlorent/claude-statusline.git
cd claude-statusline
powershell -ExecutionPolicy Bypass -File .\install-mod.ps1
```

L'installeur installe `statusline.json` dans `%USERPROFILE%\.claude` seulement si ce fichier n'existe pas, pour ne pas écraser des réglages personnels. Il copie ensuite la mod dans `skills\barre-etat` sous `.claude` et sous chaque dossier `.claude-compte*`. Quand le dossier `skills` d'un compte est une jonction vers un autre, une seule copie sert aux deux.

Fermer puis relancer Claude Code. La bande apparaît au-dessus du prompt dès l'ouverture, quotas compris, sans attendre un premier message.

Tant que l'ancienne barre est déclarée dans `settings.json`, elle reste affichée sous le prompt. C'est voulu : on compare les deux, puis on retire l'ancienne quand la mod convient.

```powershell
powershell -ExecutionPolicy Bypass -File .\install-mod.ps1 -RetirerAncienne
```

Avec cette option, le script enlève le bloc `statusLine` du `settings.json` de chaque compte après en avoir gardé une copie `settings.json.bak-statusline`.

## Vérifier la mod

`claude plugin validate "$env:USERPROFILE\.claude\skills\barre-etat"` doit répondre que la validation passe, et `claude plugin list` doit montrer `barre-etat@skills-dir` avec le statut `loaded`.

Si la seconde ligne (RAM, disque, processeur) manque alors que la première s'affiche, la compilation du programme système a échoué. Vérifier que `C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe` existe, puis supprimer `%LOCALAPPDATA%\claude-statusline\barre-etat-sys.*` pour forcer une nouvelle compilation au prochain lancement.

## Installer l'ancienne barre

```powershell
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

L'installeur fait quatre choses, dans cet ordre. Il copie `statusline.exe` dans `%USERPROFILE%\.claude\bin`. Il installe `statusline.json` s'il manque. Il garde une copie `settings.json.bak-statusline` de chaque fichier de réglages avant de le modifier. Enfin il déclare la barre dans le `settings.json` de `.claude` et de chaque dossier `.claude-compte*` trouvé, ce qui couvre tous les comptes d'un poste multi-comptes.

Pour tester le programme seul, sans Claude Code, lui envoyer des données d'exemple :

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

Tout se règle dans `%USERPROFILE%\.claude\statusline.json`, commun aux deux versions et relu à chaque rafraîchissement. Pour un terminal à fond sombre, mettre `terminal_background` à `dark` ; sans ce réglage, la mod prend la variable `STATUSLINE_BG`, à défaut le thème de Claude Code. `bar_width` change le nombre de carrés, `segments` active ou masque `ctx`, `5h`, `7d` et `model`, et `gradient` règle la force du dégradé. Le bloc `system` pilote la seconde ligne : retirer une métrique de la liste `metrics` (`ram`, `io`, `cpu`) la masque, et `"enabled": false` supprime la ligne entière. Pour utiliser un autre fichier de réglages, définir la variable d'environnement `STATUSLINE_CONFIG` avec son chemin.

## Mettre à jour

Faire `git pull` dans le dossier cloné et relancer `install-mod.ps1` (ou `install.ps1` pour l'ancienne barre). Les fichiers de la mod sont remplacés, les réglages de `statusline.json` sont conservés.

## Désinstaller

Pour la mod, supprimer le dossier `skills\barre-etat` de `%USERPROFILE%\.claude` et des dossiers `.claude-compte*` qui en ont une copie propre, puis `%LOCALAPPDATA%\claude-statusline\barre-etat-sys.exe` et `barre-etat-sys.cs`. Si l'ancienne barre avait été retirée, relancer `install.ps1` pour la remettre.

Pour l'ancienne barre, retirer le bloc `statusLine` du `settings.json` de chaque compte, ou y remettre la copie `settings.json.bak-statusline` si aucun autre réglage n'a changé depuis. Supprimer ensuite `%USERPROFILE%\.claude\bin\statusline.exe` et, si on n'en veut plus, `%USERPROFILE%\.claude\statusline.json`.

## En cas de problème

Si Windows bloque l'exécutable après un téléchargement en ZIP, ouvrir ses propriétés et cocher "Débloquer", ou lancer `Unblock-File "$env:USERPROFILE\.claude\bin\statusline.exe"`. L'exécutable n'est pas signé numériquement, ce qui peut déclencher un avertissement de SmartScreen ou de l'antivirus. La mod n'a pas ce problème, son programme système est compilé sur le poste.

Si les quotas 5 h et 7 j affichent `--%` dans l'ancienne barre, Claude Code ne les a pas transmis pour cette session ; la barre se remplit quand ils sont disponibles. Sur un poste multi-comptes, le dépôt `claude-account-menu` rafraîchit ces valeurs à chaque lancement (hook `SessionStart`). La mod lit les quotas elle-même et n'a pas besoin de ce hook. Dans les deux cas, le 5 h reste à `--%` tant que le compte n'a pas de fenêtre de 5 h ouverte, c'est-à-dire avant son premier message.

Si le script d'installation s'arrête sur une erreur de lecture de `settings.json`, le fichier contient probablement du JSON invalide (une virgule en trop, par exemple). Le corriger, puis relancer l'installeur.
