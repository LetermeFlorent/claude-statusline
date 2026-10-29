use crate::config::Rgb;
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
const CUBE_LEVELS: [i64; 6] = [0, 95, 135, 175, 215, 255];
const CUBE_BASE: i64 = 16;
const CUBE_SIDE: i64 = 6;
const GRAY_BASE: i64 = 232;
const GRAY_FIRST: i64 = 8;
const GRAY_STEP: i64 = 10;
const GRAY_COUNT: i64 = 24;

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

fn channel(value: f64) -> i64 {
    (value as i64).clamp(0, 255)
}

fn nearest_level(value: i64) -> i64 {
    (0..CUBE_SIDE)
        .min_by_key(|level| (CUBE_LEVELS[*level as usize] - value).abs())
        .unwrap_or(0)
}

fn distance(a: [i64; 3], b: [i64; 3]) -> i64 {
    (0..3).map(|i| (a[i] - b[i]) * (a[i] - b[i])).sum()
}

pub fn indexed(color: Rgb) -> u8 {
    let wanted = color.map(channel);
    let levels = wanted.map(nearest_level);
    let cube = levels.map(|level| CUBE_LEVELS[level as usize]);
    let cube_index =
        CUBE_BASE + CUBE_SIDE * CUBE_SIDE * levels[0] + CUBE_SIDE * levels[1] + levels[2];
    let mean = (wanted[0] + wanted[1] + wanted[2]) / 3;
    let step = ((mean - GRAY_FIRST + GRAY_STEP / 2) / GRAY_STEP).clamp(0, GRAY_COUNT - 1);
    let gray = GRAY_FIRST + GRAY_STEP * step;
    if distance(wanted, [gray; 3]) < distance(wanted, cube) {
        (GRAY_BASE + step) as u8
    } else {
        cube_index as u8
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

    #[test]
    fn colors_map_to_the_nearest_of_the_256_palette() {
        assert_eq!(indexed([0.0, 0.0, 0.0]), 16);
        assert_eq!(indexed([255.0, 255.0, 255.0]), 231);
        assert_eq!(indexed([255.0, 0.0, 0.0]), 196);
        assert_eq!(indexed([0.0, 0.0, 255.0]), 21);
        assert_eq!(indexed([128.0, 128.0, 128.0]), 244);
        assert_eq!(indexed([85.0, 89.0, 95.0]), 240);
        assert_eq!(indexed([300.0, -4.0, 0.0]), 196);
    }
}
