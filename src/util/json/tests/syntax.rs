use crate::json::{parse, Syntax, Value};

#[test]
fn tolerant_syntax_reads_loose_json() {
    let parsed = parse(
        "{\"a\":1,\"a\":2,\"s\":\"\\u00e9\\ud83d\\ude00\"}",
        Syntax::Tolerant,
    )
    .unwrap();
    assert_eq!(parsed.get("a").unwrap().num(), Some(2.0));
    assert_eq!(parsed.get("s").unwrap().str(), Some("\u{e9}\u{1f600}"));
    assert!(parse("{,\"a\":1,,}", Syntax::Tolerant).is_some());
    assert!(
        parse("[1,,2,]", Syntax::Tolerant)
            .unwrap()
            .arr()
            .unwrap()
            .len()
            == 2
    );
    assert!(parse("{\"a\" 1}", Syntax::Tolerant).is_none());
    assert!(parse("{\"a\":1, // x\n}", Syntax::Commented).is_some());
    assert!(parse("\u{feff}{\"a\":1}", Syntax::Tolerant).is_some());
    assert_eq!(
        parse("{\"s\":\"a\\ud83db\"}", Syntax::Tolerant)
            .unwrap()
            .get("s")
            .unwrap()
            .str(),
        Some("ab")
    );
    assert!(parse("[1.]", Syntax::Tolerant).is_none());
    assert_eq!(
        parse("{\"x\":1} reste", Syntax::Tolerant)
            .unwrap()
            .get("x")
            .unwrap()
            .num(),
        Some(1.0)
    );
}

#[test]
fn strict_syntax_needs_single_commas_and_nothing_after() {
    for text in [
        "{\"a\":1}{\"b\":2}",
        "{\"a\":1} x",
        "{\"a\":1 \"b\":2}",
        "[1 2]",
        "[1,,2]",
        "{,\"a\":1}",
        "[1,]",
        "{\"a\":1,}",
        "[,]",
        "{\"a\":1} // x",
        "[\"a\\ud83db\"]",
        "[\"\\ude00\"]",
    ] {
        assert!(parse(text, Syntax::Strict).is_none(), "{}", text);
    }
    let ok = parse(" {\"a\": [1, 2], \"b\": {}, \"c\": []} \n", Syntax::Strict).unwrap();
    assert_eq!(ok.get("a").and_then(Value::arr).map(|a| a.len()), Some(2));
    assert!(parse("\u{feff}[\"\\ud83d\\ude00\"]", Syntax::Strict).is_some());
}

#[test]
fn out_of_range_numbers_are_refused() {
    assert!(parse("{\"a\": 1e999}", Syntax::Tolerant).is_none());
    assert!(parse("[-1e999]", Syntax::Tolerant).is_none());
    let tiny = parse("[1e-999]", Syntax::Tolerant).unwrap();
    assert_eq!(tiny.arr().unwrap()[0].num(), Some(0.0));
}
