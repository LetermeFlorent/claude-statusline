use crate::json::{parse, Syntax};

fn text_of(json: &str, syntax: Syntax) -> Option<String> {
    let value = parse(json, syntax)?;
    value.at(&["s"])?.str().map(str::to_string)
}

fn strict_read(escapes: &str) -> Option<String> {
    text_of(&format!("{{\"s\":\"{escapes}\"}}"), Syntax::Strict)
}

#[test]
fn simple_escapes_decode_to_their_characters() {
    assert_eq!(strict_read("\\\""), Some("\"".to_string()));
    assert_eq!(strict_read("\\\\"), Some("\\".to_string()));
    assert_eq!(strict_read("\\/"), Some("/".to_string()));
    assert_eq!(strict_read("\\b"), Some("\u{8}".to_string()));
    assert_eq!(strict_read("\\f"), Some("\u{c}".to_string()));
    assert_eq!(strict_read("\\n"), Some("\n".to_string()));
    assert_eq!(strict_read("\\r"), Some("\r".to_string()));
    assert_eq!(strict_read("\\t"), Some("\t".to_string()));
}

#[test]
fn unicode_escapes_and_surrogate_pairs_decode() {
    assert_eq!(strict_read("\\u0041\\u00E9"), Some("A\u{e9}".to_string()));
    assert_eq!(strict_read("\\uD83D\\uDE00"), Some("\u{1f600}".to_string()));
    assert_eq!(
        strict_read("a\\ud800\\udc00b"),
        Some("a\u{10000}b".to_string())
    );
}

#[test]
fn broken_surrogates_are_dropped_and_refused_by_strict_syntax() {
    let json = |e: &str| format!("{{\"s\":\"x{e}y\"}}");
    for e in ["\\ud83d", "\\ud83dz", "\\ud83d\\u0041", "\\ude00"] {
        assert!(parse(&json(e), Syntax::Strict).is_none());
    }
    let lone = text_of(&json("\\ud83dz"), Syntax::Tolerant);
    assert_eq!(lone, Some("xzy".to_string()));
    let rewound = text_of(&json("\\ud83d\\u0041"), Syntax::Tolerant);
    assert_eq!(rewound, Some("xAy".to_string()));
    let low = text_of(&json("\\ude00"), Syntax::Tolerant);
    assert_eq!(low, Some("xy".to_string()));
    assert!(parse(&json("\\ud83d\\u00"), Syntax::Tolerant).is_none());
}

#[test]
fn invalid_escapes_and_unterminated_strings_fail() {
    for bad in ["\\x", "\\u12", "\\u12g4", "\\"] {
        let json = format!("{{\"s\":\"{bad}\"}}");
        assert!(parse(&json, Syntax::Tolerant).is_none());
    }
    for open in ["{\"s\":\"abc", "{\"s\":\"abc\\", "{\"s\":\"\\u00e9"] {
        assert!(parse(open, Syntax::Tolerant).is_none());
    }
}
