use super::common::{fitted, line, raw, sample};
use super::helpers::glyphs;
use crate::render::screen::fit;
use crate::render::Depth;

#[test]
fn the_line_fits_in_80_columns() {
    let wide = glyphs(&line(false, None, &sample()));
    assert!(fit::widest(&wide) > 80, "{}", wide);
    let narrow = glyphs(&fitted(80, None));
    assert!(fit::widest(&narrow) <= 80, "{}", narrow);
}

#[test]
fn the_reset_times_go_first_then_the_effort_then_the_bar_width() {
    let text = |columns| glyphs(&fitted(columns, None));
    let bar = "\u{25A0}".repeat(8);
    assert!(text(85).contains("1h00"), "{}", text(85));
    let no_resets = text(84);
    assert!(!no_resets.contains("1h00"), "{}", no_resets);
    assert!(no_resets.contains(":high"), "{}", no_resets);
    let no_effort = text(70);
    assert!(!no_effort.contains(":high"), "{}", no_effort);
    assert!(no_effort.contains("Opus 4"), "{}", no_effort);
    assert!(no_effort.contains(&bar), "{}", no_effort);
    let narrow = text(60);
    assert!(!narrow.contains(&bar), "{}", narrow);
    assert!(fit::widest(&narrow) <= 60, "{}", narrow);
}

#[test]
fn a_bar_width_of_4_fits_in_80_columns_with_every_detail() {
    let text = glyphs(&fitted(80, Some("{\"bar_width\": 4}")));
    assert!(text.contains("1h00") && text.contains(":high"), "{}", text);
    assert!(fit::widest(&text) <= 80, "{}", text);
}

#[test]
fn without_truecolor_only_256_color_sequences_are_sent() {
    let indexed = raw(Depth::Indexed);
    assert!(!indexed.contains("38;2;"), "{:?}", indexed);
    assert!(indexed.contains("[38;5;240m"), "{:?}", indexed);
    let colors = indexed.matches("38;5;").count();
    assert_eq!(colors, indexed.matches("[38;").count());
    let true_color = raw(Depth::TrueColor);
    assert!(!true_color.contains("38;5;"), "{:?}", true_color);
    assert!(true_color.contains("38;2;85;89;95"), "{:?}", true_color);
}
