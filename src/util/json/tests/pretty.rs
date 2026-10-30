use crate::json::{parse, pretty, Syntax, Value};

#[test]
fn pretty_keeps_key_order_and_number_text() {
    let document = parse(
        "{\"b\":[1,2.50],\"a\":{},\"c\":\"x\\u0001\\\"\",\"d\":[]}",
        Syntax::Tolerant,
    )
    .unwrap();
    assert_eq!(pretty(&document), "{\n  \"b\": [\n    1,\n    2.50\n  ],\n  \"a\": {},\n  \"c\": \"x\\u0001\\\"\",\n  \"d\": []\n}\n");
    let mut document = parse("{\"a\":1,\"b\":2,\"a\":3}", Syntax::Tolerant).unwrap();
    document.set("a", Value::Bool(true));
    document.set("c", Value::Null);
    assert_eq!(
        pretty(&document),
        "{\n  \"a\": true,\n  \"b\": 2,\n  \"c\": null\n}\n"
    );
}
