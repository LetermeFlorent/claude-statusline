mod check;
mod help;
mod language;
mod table;

pub use check::*;
pub use help::HELP;
pub use table::*;

use alloc::string::String;
use language::Language;

const PLACEHOLDER: &str = "{}";

pub struct Text {
    english: &'static str,
    french: &'static str,
}

impl Text {
    pub const fn new(english: &'static str, french: &'static str) -> Text {
        Text { english, french }
    }

    pub fn text(&self) -> &'static str {
        self.in_language(language::current())
    }

    pub fn with(&self, values: &[&str]) -> String {
        fill(self.text(), values)
    }

    fn in_language(&self, language: Language) -> &'static str {
        match language {
            Language::English => self.english,
            Language::French => self.french,
        }
    }
}

fn fill(template: &str, values: &[&str]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut values = values.iter();
    let mut rest = template;
    while let Some(at) = rest.find(PLACEHOLDER) {
        out.push_str(&rest[..at]);
        out.push_str(values.next().copied().unwrap_or_default());
        rest = &rest[at + PLACEHOLDER.len()..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_fill_the_placeholders_in_order() {
        assert_eq!(
            fill("{} -> {}\n", &["2.1.0", "v2.2.0"]),
            "2.1.0 -> v2.2.0\n"
        );
    }

    #[test]
    fn braces_inside_a_value_stay_as_they_are() {
        assert_eq!(fill("{} and {}", &["C:/{}", "x"]), "C:/{} and x");
    }

    #[test]
    fn a_missing_value_leaves_its_placeholder_empty() {
        assert_eq!(fill("a{}b{}c", &["1"]), "a1bc");
    }

    #[test]
    fn each_language_gets_its_own_text() {
        let text = Text::new("up to date", "à jour");
        assert_eq!(text.in_language(Language::English), "up to date");
        assert_eq!(text.in_language(Language::French), "à jour");
    }

    #[test]
    fn the_help_names_the_account_folder() {
        for help in [HELP.english, HELP.french] {
            assert_eq!(help.matches("$CLAUDE_CONFIG_DIR").count(), 2, "{}", help);
        }
    }

    #[test]
    fn both_languages_take_the_same_values_and_line_ending() {
        let placeholders = |s: &str| s.matches(PLACEHOLDER).count();
        for text in ALL.into_iter().chain(CHECK_TEXTS).chain([&HELP]) {
            assert!(!text.english.is_empty(), "{}", text.french);
            assert!(!text.french.is_empty(), "{}", text.english);
            assert_eq!(
                placeholders(text.english),
                placeholders(text.french),
                "{}",
                text.english
            );
            assert_eq!(
                text.english.ends_with('\n'),
                text.french.ends_with('\n'),
                "{}",
                text.english
            );
        }
    }
}
