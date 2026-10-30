const BYTE_ORDER_MARK: char = '\u{feff}';

pub fn line_and_column(text: &str, at: usize) -> (usize, usize) {
    let mut at = at.min(text.len());
    while !text.is_char_boundary(at) {
        at -= 1;
    }
    let before = &text[..at];
    let line_start = before.rfind('\n').map_or(0, |newline| newline + 1);
    let column = before[line_start..]
        .chars()
        .filter(|&c| c != BYTE_ORDER_MARK)
        .count();
    (before.matches('\n').count() + 1, column + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn columns_count_characters_from_one() {
        let text = "\u{feff}{\"\u{e9}\": 1,\r\n \"x\": 2}";
        assert_eq!(line_and_column(text, 3), (1, 1));
        assert_eq!(line_and_column(text, text.find('1').unwrap()), (1, 7));
        assert_eq!(line_and_column(text, text.find('x').unwrap()), (2, 3));
    }

    #[test]
    fn an_offset_inside_a_character_points_at_that_character() {
        let text = "\"\u{e9}\"";
        assert_eq!(line_and_column(text, 2), (1, 2));
        assert_eq!(line_and_column(text, 99), (1, 4));
    }
}
