use super::Text;

pub const BENCH_SUMMARY: Text = Text::new(
    "{} renders, {} ms per render ({} bytes of output)\n",
    "{} rendus, {} ms par rendu ({} octets produits)\n",
);
pub const BENCH_STATE: Text = Text::new("state", "état");
pub const BENCH_CONFIG: Text = Text::new("config", "config");
pub const BENCH_SESSION: Text = Text::new("session", "session");
pub const BENCH_RENDER: Text = Text::new("render", "rendu");
pub const BENCH_WRITE: Text = Text::new("write", "écriture");
pub const UNKNOWN_ARGUMENT: Text =
    Text::new("unknown argument: {}\n{}", "argument inconnu : {}\n{}");
pub const BAD_BENCH_COUNT: Text = Text::new(
    "--bench expects a whole number above 0, got: {}\n",
    "--bench attend un nombre entier supérieur à 0, reçu : {}\n",
);
pub const SETTINGS_UNREADABLE: Text = Text::new(
    "settings cannot be read, nothing changed: {}\n",
    "réglages illisibles, rien n'est modifié : {}\n",
);
pub const CANNOT_WRITE: Text = Text::new("cannot write {}\n", "écriture impossible : {}\n");
pub const BACKUP: Text = Text::new("backup: {}\n", "sauvegarde : {}\n");
pub const GLOBAL_REPAIRED: Text = Text::new(
    "global statusLine repaired in {} (was: {})\n",
    "statusLine globale réparée dans {} (était : {})\n",
);
pub const GLOBAL_KEPT: Text = Text::new(
    "global file left as is: {}\n",
    "fichier global laissé tel quel : {}\n",
);
pub const NO_HOME: Text = Text::new("home folder not found\n", "dossier personnel introuvable\n");
pub const NO_EXE: Text = Text::new(
    "path of this program not found\n",
    "chemin du binaire introuvable\n",
);
pub const INSTALLED: Text = Text::new("statusLine set in {}\n", "statusLine posée dans {}\n");
pub const CONFIG_WRITTEN: Text = Text::new("configuration: {}\n", "configuration : {}\n");
pub const RESTART: Text = Text::new(
    "restart Claude Code to see the bar\n",
    "relancez Claude Code pour voir la barre\n",
);
pub const NOTHING_TO_REMOVE: Text =
    Text::new("no statusLine to remove\n", "aucune statusLine à retirer\n");
pub const REMOVED: Text = Text::new("statusLine removed from {}\n", "statusLine retirée de {}\n");
pub const NO_RELEASE: Text = Text::new(
    "latest release not found\n",
    "dernière version introuvable\n",
);
pub const CURL_MISSING: Text = Text::new(
    "curl.exe not found in the system folder\n",
    "curl.exe introuvable dans le dossier système\n",
);
pub const RATE_LIMITED: Text = Text::new(
    "GitHub rate limit reached, try again later\n",
    "limite de requêtes GitHub atteinte, réessayez plus tard\n",
);
pub const NO_ASSET: Text = Text::new(
    "no released binary for this system\n",
    "aucun binaire publié pour ce système\n",
);
pub const UPDATE_FAILED: Text =
    Text::new("update to {} failed\n", "mise à jour vers {} impossible\n");
pub const UP_TO_DATE: Text = Text::new("already up to date: {}\n", "déjà à jour : {}\n");
pub const UPDATED: Text = Text::new("updated: {} -> {}\n", "mise à jour : {} -> {}\n");
pub const STATE_DIR_FAILED: Text = Text::new(
    "cannot create the state folder: {}\n",
    "dossier d'état impossible à créer : {}\n",
);
pub const NO_STATE_DIR: Text =
    Text::new("state folder not found\n", "dossier d'état introuvable\n");
pub const UPDATE_RUNNING: Text = Text::new(
    "an update is already running\n",
    "mise à jour déjà en cours\n",
);

#[cfg(test)]
pub const ALL: [&Text; 30] = [
    &BENCH_SUMMARY,
    &BENCH_STATE,
    &BENCH_CONFIG,
    &BENCH_SESSION,
    &BENCH_RENDER,
    &BENCH_WRITE,
    &UNKNOWN_ARGUMENT,
    &BAD_BENCH_COUNT,
    &SETTINGS_UNREADABLE,
    &CANNOT_WRITE,
    &BACKUP,
    &GLOBAL_REPAIRED,
    &GLOBAL_KEPT,
    &NO_HOME,
    &NO_EXE,
    &INSTALLED,
    &CONFIG_WRITTEN,
    &RESTART,
    &NOTHING_TO_REMOVE,
    &REMOVED,
    &NO_RELEASE,
    &CURL_MISSING,
    &RATE_LIMITED,
    &NO_ASSET,
    &UPDATE_FAILED,
    &UP_TO_DATE,
    &UPDATED,
    &STATE_DIR_FAILED,
    &NO_STATE_DIR,
    &UPDATE_RUNNING,
];
