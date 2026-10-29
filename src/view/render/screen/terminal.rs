use crate::os;
use alloc::string::String;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Depth {
    TrueColor,
    Indexed,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Terminal {
    pub columns: Option<usize>,
    pub depth: Depth,
}

const TRUE_COLOR_WORDS: [&str; 2] = ["truecolor", "24bit"];

impl Terminal {
    #[cfg(test)]
    pub const WIDE: Terminal = Terminal {
        columns: None,
        depth: Depth::TrueColor,
    };

    pub fn current() -> Terminal {
        Terminal::detect(&os::env, cfg!(windows))
    }

    pub fn detect(
        variable: &dyn Fn(&str) -> Option<String>,
        console_draws_24_bit: bool,
    ) -> Terminal {
        let announced = variable("COLORTERM").is_some_and(|word| {
            let word = word.trim().to_lowercase();
            TRUE_COLOR_WORDS.contains(&word.as_str())
        });
        let known = announced || console_draws_24_bit || variable("WT_SESSION").is_some();
        let columns = variable("COLUMNS")
            .and_then(|text| text.trim().parse::<usize>().ok())
            .filter(|columns| *columns > 0);
        Terminal {
            columns,
            depth: if known {
                Depth::TrueColor
            } else {
                Depth::Indexed
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terminal(set: &[(&str, &str)], console: bool) -> Terminal {
        Terminal::detect(
            &|name| {
                set.iter()
                    .find(|(key, _)| *key == name)
                    .map(|(_, value)| String::from(*value))
            },
            console,
        )
    }

    #[test]
    fn without_colorterm_the_colors_fall_back_to_256() {
        assert_eq!(terminal(&[], false).depth, Depth::Indexed);
        let xterm = [("TERM", "xterm-256color")];
        assert_eq!(terminal(&xterm, false).depth, Depth::Indexed);
        assert_eq!(
            terminal(&[("COLORTERM", "yes")], false).depth,
            Depth::Indexed
        );
    }

    #[test]
    fn colorterm_truecolor_or_24bit_keeps_24_bit_colors() {
        for word in ["truecolor", "24bit", "TrueColor", " 24BIT "] {
            assert_eq!(
                terminal(&[("COLORTERM", word)], false).depth,
                Depth::TrueColor,
                "{}",
                word
            );
        }
    }

    #[test]
    fn windows_terminal_and_windows_consoles_keep_24_bit_colors() {
        assert_eq!(
            terminal(&[("WT_SESSION", "x")], false).depth,
            Depth::TrueColor
        );
        assert_eq!(terminal(&[], true).depth, Depth::TrueColor);
    }

    #[test]
    fn columns_count_only_when_they_are_a_positive_number() {
        assert_eq!(terminal(&[("COLUMNS", "80")], false).columns, Some(80));
        assert_eq!(terminal(&[("COLUMNS", " 132 ")], false).columns, Some(132));
        for text in ["0", "-5", "wide", "", "8.5"] {
            assert_eq!(
                terminal(&[("COLUMNS", text)], false).columns,
                None,
                "{}",
                text
            );
        }
        assert_eq!(terminal(&[], false).columns, None);
    }
}
