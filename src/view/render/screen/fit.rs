use crate::plain;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Layout {
    pub resets: bool,
    pub effort: bool,
    pub width: usize,
}

impl Layout {
    pub fn full(width: usize) -> Layout {
        Layout {
            resets: true,
            effort: true,
            width,
        }
    }

    pub fn narrower(self) -> Option<Layout> {
        if self.resets {
            return Some(Layout {
                resets: false,
                ..self
            });
        }
        if self.effort {
            return Some(Layout {
                effort: false,
                ..self
            });
        }
        if self.width > 1 {
            return Some(Layout {
                width: self.width - 1,
                ..self
            });
        }
        None
    }
}

pub fn widest(text: &str) -> usize {
    plain::strip(text)
        .lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layout_gives_up_the_resets_then_the_effort_then_the_width() {
        let mut layout = Layout::full(3);
        let mut seen = alloc::vec![];
        while let Some(next) = layout.narrower() {
            seen.push((next.resets, next.effort, next.width));
            layout = next;
        }
        assert_eq!(
            seen,
            [
                (false, true, 3),
                (false, false, 3),
                (false, false, 2),
                (false, false, 1)
            ]
        );
    }

    #[test]
    fn the_width_counts_visible_characters_of_the_longest_line() {
        let text = "\x1b[1m\x1b[38;2;1;2;3mabc\x1b[0m\nabcdef\x1b[0m\n";
        assert_eq!(widest(text), 6);
        assert_eq!(widest(""), 0);
    }
}
