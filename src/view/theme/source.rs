use crate::config::background_word;
use alloc::format;
use alloc::string::String;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Source {
    Environment,
    Config,
    ColorFgBg,
    Vscode,
    WindowsTerminal,
    Zed,
    Ghostty,
    System,
    Assumed,
    Saved,
}

const NAMES: [(Source, &str); 10] = [
    (Source::Environment, "STATUSLINE_BG"),
    (Source::Config, "terminal_background"),
    (Source::ColorFgBg, "COLORFGBG"),
    (Source::Vscode, "vscode"),
    (Source::WindowsTerminal, "windows-terminal"),
    (Source::Zed, "zed"),
    (Source::Ghostty, "ghostty"),
    (Source::System, "system theme"),
    (Source::Assumed, "unknown terminal, dark assumed"),
    (Source::Saved, "saved choice"),
];

impl Source {
    pub fn name(self) -> &'static str {
        NAMES
            .iter()
            .find(|(source, _)| *source == self)
            .map_or("", |(_, name)| name)
    }

    pub fn from_name(text: &str) -> Source {
        NAMES
            .iter()
            .find(|(_, name)| *name == text)
            .map_or(Source::Saved, |(source, _)| *source)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Decision {
    pub light: bool,
    pub source: Source,
}

impl Decision {
    pub fn chosen(
        background: Option<&str>,
        light_override: Option<bool>,
        colorfgbg: Option<&str>,
    ) -> Option<Decision> {
        let decision = |light, source| Some(Decision { light, source });
        if let Some(light) = background.and_then(background_word) {
            return decision(light, Source::Environment);
        }
        if let Some(light) = light_override {
            return decision(light, Source::Config);
        }
        colorfgbg
            .and_then(super::colorfgbg)
            .and_then(|light| decision(light, Source::ColorFgBg))
    }

    pub fn report(self) -> String {
        let word = if self.light { "light" } else { "dark" };
        format!("{}\nsource: {}\n", word, self.source.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_report_starts_with_the_background_and_names_its_source() {
        let decision = Decision {
            light: false,
            source: Source::Environment,
        };
        assert_eq!(decision.report(), "dark\nsource: STATUSLINE_BG\n");
        let decision = Decision {
            light: true,
            source: Source::System,
        };
        assert_eq!(decision.report(), "light\nsource: system theme\n");
    }

    fn chosen(
        background: Option<&str>,
        light_override: Option<bool>,
        colorfgbg: Option<&str>,
    ) -> Option<(bool, Source)> {
        Decision::chosen(background, light_override, colorfgbg).map(|d| (d.light, d.source))
    }

    #[test]
    fn a_forced_background_names_the_setting_that_forced_it() {
        let env = Some("dark");
        assert_eq!(
            chosen(env, Some(true), Some("0;15")),
            Some((false, Source::Environment))
        );
        assert_eq!(
            chosen(None, Some(true), Some("15;0")),
            Some((true, Source::Config))
        );
        assert_eq!(
            chosen(None, None, Some("0;15")),
            Some((true, Source::ColorFgBg))
        );
    }

    #[test]
    fn nothing_is_chosen_when_no_setting_speaks() {
        assert_eq!(chosen(None, None, None), None);
        assert_eq!(chosen(Some("blue"), None, Some("0;-1")), None);
    }

    #[test]
    fn every_source_survives_a_trip_through_its_name() {
        for (source, name) in NAMES {
            assert_eq!(Source::from_name(name), source);
        }
        assert_eq!(Source::from_name("something else"), Source::Saved);
    }

    #[test]
    fn source_names_are_unique_and_printable() {
        for (index, (_, name)) in NAMES.iter().enumerate() {
            assert!(name.chars().all(|c| !c.is_control()), "{}", name);
            assert!(NAMES[index + 1..].iter().all(|(_, other)| other != name));
        }
    }
}
