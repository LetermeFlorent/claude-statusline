use alloc::string::String;
use core::ops::RangeInclusive;

const ESCAPE: char = '\x1b';
const SEQUENCE_START: char = '[';
const SEQUENCE_END: RangeInclusive<char> = '\x40'..='\x7e';
const DUMB_TERMINAL: &str = "dumb";

pub fn wanted(variable: &dyn Fn(&str) -> Option<String>) -> bool {
    let no_color = variable("NO_COLOR").is_some_and(|value| !value.is_empty());
    no_color || variable("TERM").as_deref() == Some(DUMB_TERMINAL)
}

pub fn strip(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c != ESCAPE {
            out.push(c);
            continue;
        }
        if chars.next_if_eq(&SEQUENCE_START).is_some() {
            chars.by_ref().find(|c| SEQUENCE_END.contains(c));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn variables(set: &[(&str, &str)]) -> bool {
        wanted(&|name| {
            set.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| String::from(*value))
        })
    }

    #[test]
    fn no_color_with_a_value_asks_for_plain_text() {
        assert!(variables(&[("NO_COLOR", "1")]));
        assert!(variables(&[("NO_COLOR", "0"), ("TERM", "xterm-256color")]));
    }

    #[test]
    fn an_empty_no_color_is_ignored() {
        assert!(!variables(&[("NO_COLOR", "")]));
    }

    #[test]
    fn a_dumb_terminal_asks_for_plain_text() {
        assert!(variables(&[("TERM", "dumb")]));
        assert!(!variables(&[("TERM", "xterm-256color")]));
        assert!(!variables(&[]));
    }

    #[test]
    fn styles_and_colors_are_removed() {
        let line = "\x1b[2m\x1b[38;2;232;234;237m\u{aa}\x1b[0m\x1b[1m\x1b[38;2;232;234;237mctx\x1b[0m \x1b[4mOpus\x1b[0m\n";
        assert_eq!(strip(line), "\u{aa}ctx Opus\n");
    }

    #[test]
    fn text_without_escapes_is_left_alone() {
        let line = "ram \u{25a0}\u{25aa} 7.9/15.3G | cpu 23%\n";
        assert_eq!(strip(line), line);
    }

    #[test]
    fn a_stray_or_cut_escape_leaves_no_trace() {
        assert_eq!(strip("a\x1bb"), "ab");
        assert_eq!(strip("a\x1b[38;2"), "a");
    }
}
