use crate::os;
use alloc::string::String;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Terminal {
    pub columns: Option<usize>,
}

impl Terminal {
    #[cfg(test)]
    pub const WIDE: Terminal = Terminal { columns: None };

    pub fn current() -> Terminal {
        Terminal::detect(&os::env)
    }

    pub fn detect(variable: &dyn Fn(&str) -> Option<String>) -> Terminal {
        let columns = variable("COLUMNS")
            .and_then(|text| text.trim().parse::<usize>().ok())
            .filter(|columns| *columns > 0);
        Terminal { columns }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terminal(set: &[(&str, &str)]) -> Terminal {
        Terminal::detect(&|name| {
            set.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| String::from(*value))
        })
    }

    #[test]
    fn columns_count_only_when_they_are_a_positive_number() {
        assert_eq!(terminal(&[("COLUMNS", "80")]).columns, Some(80));
        assert_eq!(terminal(&[("COLUMNS", " 132 ")]).columns, Some(132));
        for text in ["0", "-5", "wide", "", "8.5"] {
            assert_eq!(terminal(&[("COLUMNS", text)]).columns, None, "{}", text);
        }
        assert_eq!(terminal(&[]).columns, None);
    }
}
