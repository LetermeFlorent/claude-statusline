use super::Mode;

const LIGHT_WORDS: [&str; 4] = ["light", "latte", "dawn", "day"];
const NOT_LIGHT: [&str; 2] = ["moonlight", "twilight"];
const DARK_WORDS: [&str; 6] = ["dark", "night", "black", "moon", "mocha", "macchiato"];

pub fn word_mode(name: &str) -> Option<Mode> {
    let lower = name.to_lowercase();
    let rest = NOT_LIGHT
        .iter()
        .fold(lower.clone(), |text, word| text.replace(word, " "));
    if LIGHT_WORDS.iter().any(|word| rest.contains(word)) {
        Some(Mode::Light)
    } else if DARK_WORDS
        .iter()
        .chain(&NOT_LIGHT)
        .any(|word| lower.contains(word))
    {
        Some(Mode::Dark)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_words_count_unless_they_belong_to_a_dark_word() {
        assert_eq!(word_mode("Catppuccin Latte"), Some(Mode::Light));
        assert_eq!(word_mode("Rosé Pine Dawn"), Some(Mode::Light));
        assert_eq!(word_mode("Tokyo Night Day"), Some(Mode::Light));
        assert_eq!(word_mode("Night Owl Light"), Some(Mode::Light));
        assert_eq!(word_mode("Moonlight II"), Some(Mode::Dark));
        assert_eq!(word_mode("Twilight"), Some(Mode::Dark));
        assert_eq!(word_mode("Catppuccin Mocha"), Some(Mode::Dark));
        assert_eq!(word_mode("Monokai"), None);
    }
}
