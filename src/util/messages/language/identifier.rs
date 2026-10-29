use super::Language;

pub fn from_language_id(id: u16) -> Language {
    const PRIMARY_LANGUAGE_MASK: u16 = 0x3ff;
    const LANG_FRENCH: u16 = 0x0c;
    if id & PRIMARY_LANGUAGE_MASK == LANG_FRENCH {
        Language::French
    } else {
        Language::English
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn french_windows_display_languages_pick_french() {
        assert_eq!(from_language_id(0x040c), Language::French);
        assert_eq!(from_language_id(0x0c0c), Language::French);
        assert_eq!(from_language_id(0x100c), Language::French);
    }

    #[test]
    fn other_windows_display_languages_pick_english() {
        assert_eq!(from_language_id(0x0409), Language::English);
        assert_eq!(from_language_id(0x0407), Language::English);
        assert_eq!(from_language_id(0x0462), Language::English);
        assert_eq!(from_language_id(0), Language::English);
    }
}
