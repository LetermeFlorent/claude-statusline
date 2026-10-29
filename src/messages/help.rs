use super::Text;

pub const HELP: Text = Text::new(
    "statusline - status line for Claude Code

With no argument, reads the Claude Code JSON on standard input and writes the
bar on standard output.

  --install     adds the statusLine entry to settings.json in the account
                folder, $CLAUDE_CONFIG_DIR or else ~/.claude
  --uninstall   removes it
  --demo        prints a sample bar, without Claude Code
  --bench [n]   times the work alone, process start excluded
  --theme       background detected for the terminal: light or dark
  --check       lists the mistakes in the settings file, with their line
  --update      installs the latest release published on GitHub
  --version     version number
  --help        this help

Settings: $STATUSLINE_CONFIG, else statusline.json in $CLAUDE_CONFIG_DIR
when it is there, else ~/.claude/statusline.json
",
    "statusline - ligne de statut pour Claude Code

Sans argument, lit le JSON de Claude Code sur l'entrée standard et écrit la
barre sur la sortie standard.

  --install     pose l'entrée statusLine dans le settings.json du dossier
                du compte, $CLAUDE_CONFIG_DIR ou sinon ~/.claude
  --uninstall   la retire
  --demo        affiche un rendu d'exemple, sans Claude Code
  --bench [n]   coût du travail seul, lancement du processus exclu
  --theme       fond détecté pour le terminal : light ou dark
  --check       liste les erreurs du fichier de réglages, avec leur ligne
  --update      installe la dernière version publiée sur GitHub
  --version     numéro de version
  --help        cette aide

Réglages : $STATUSLINE_CONFIG, sinon statusline.json dans $CLAUDE_CONFIG_DIR
s'il existe, sinon ~/.claude/statusline.json
",
);
