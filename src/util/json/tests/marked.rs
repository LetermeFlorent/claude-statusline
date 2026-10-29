use crate::json::{parse_marked, Syntax};

#[test]
fn marked_parse_gives_the_offset_of_every_key_and_value() {
    let text = "{\"a\": [1, {\"b\": true}], \"c\": null} // x";
    let marked = parse_marked(text, Syntax::Commented).ok().unwrap();
    let spans = &marked.spans;
    let list = &spans.inner[0];
    assert_eq!((spans.at, spans.keys.as_slice()), (0, [1, 24].as_slice()));
    assert_eq!((list.at, list.inner[0].at, list.inner[1].at), (6, 7, 10));
    assert_eq!(list.inner[1].keys, [11]);
    assert_eq!(list.inner[1].inner[0].at, 16);
    assert_eq!(spans.inner[1].at, 29);
    assert_eq!(marked.end, text.len());
    let after_mark = parse_marked("\u{feff} {}", Syntax::Commented).ok().unwrap();
    assert_eq!(after_mark.spans.at, 4);
}

#[test]
fn marked_parse_stops_where_the_text_goes_wrong() {
    let stop = |text: &str| parse_marked(text, Syntax::Commented).err();
    assert_eq!(stop("{\"a\": tru}"), Some(6));
    assert_eq!(stop("{\"a\" 1}"), Some(5));
    assert_eq!(stop("{\"a\": 1"), Some(7));
    assert_eq!(stop(""), Some(0));
    let trailing = parse_marked("{} }", Syntax::Commented).ok().unwrap();
    assert_eq!(trailing.end, 3);
}
