use super::Text;

pub const CHECK_CLEAN: Text = Text::new("{}: no mistake found\n", "{} : aucune erreur trouvée\n");
pub const CHECK_MISSING: Text = Text::new(
    "{}: no such file, every setting keeps its default\n",
    "{} : fichier absent, chaque réglage garde sa valeur par défaut\n",
);
pub const CHECK_UNREADABLE: Text = Text::new("cannot read {}\n", "lecture impossible : {}\n");
pub const INVALID_JSON: Text = Text::new(
    "invalid JSON, every setting falls back to its default\n",
    "JSON invalide, chaque réglage reprend sa valeur par défaut\n",
);
pub const NOT_AN_OBJECT: Text = Text::new(
    "the file must hold one object in braces, every setting falls back to its default\n",
    "le fichier doit contenir un objet entre accolades, chaque réglage reprend sa valeur par défaut\n",
);
pub const TRAILING_TEXT: Text = Text::new(
    "text after the closing brace, ignored\n",
    "texte après l'accolade fermante, ignoré\n",
);
pub const UNKNOWN_KEY: Text = Text::new(
    "unknown key \"{}\", ignored\n",
    "clé inconnue \"{}\", ignorée\n",
);
pub const EXPECTS: Text = Text::new("\"{}\" expects {}\n", "\"{}\" attend {}\n");
pub const EXPECTED_FLAG: Text = Text::new("true or false", "true ou false");
pub const EXPECTED_NUMBER: Text = Text::new("a number", "un nombre");
pub const EXPECTED_TEXT: Text = Text::new("a text in quotes", "un texte entre guillemets");
pub const EXPECTED_COLOR: Text = Text::new(
    "a color such as \"#1E787D\"",
    "une couleur comme \"#1E787D\"",
);
pub const EXPECTED_LIST: Text = Text::new("a list in square brackets", "une liste entre crochets");
pub const EXPECTED_OBJECT: Text = Text::new("an object in braces", "un objet entre accolades");
pub const EXPECTED_ONE_OF: Text = Text::new("one of: {}", "une valeur parmi : {}");

#[cfg(test)]
pub const CHECK_TEXTS: [&Text; 15] = [
    &CHECK_CLEAN,
    &CHECK_MISSING,
    &CHECK_UNREADABLE,
    &INVALID_JSON,
    &NOT_AN_OBJECT,
    &TRAILING_TEXT,
    &UNKNOWN_KEY,
    &EXPECTS,
    &EXPECTED_FLAG,
    &EXPECTED_NUMBER,
    &EXPECTED_TEXT,
    &EXPECTED_COLOR,
    &EXPECTED_LIST,
    &EXPECTED_OBJECT,
    &EXPECTED_ONE_OF,
];
