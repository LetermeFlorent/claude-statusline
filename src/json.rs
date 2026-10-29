use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64, String),
    Str(String),
    Arr(Vec<Value>),
    Obj(Vec<(String, Value)>),
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Obj(m) => m.iter().rev().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn at(&self, path: &[&str]) -> Option<&Value> {
        let mut v = self;
        for k in path {
            v = v.get(k)?;
        }
        Some(v)
    }

    pub fn str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn num(&self) -> Option<f64> {
        match self {
            Value::Num(n, _) => Some(*n),
            _ => None,
        }
    }

    pub fn bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn arr(&self) -> Option<&[Value]> {
        match self {
            Value::Arr(a) => Some(a),
            _ => None,
        }
    }

    pub fn is_obj(&self) -> bool {
        matches!(self, Value::Obj(_))
    }

    pub fn set(&mut self, key: &str, val: Value) {
        let Value::Obj(members) = self else {
            return;
        };
        let Some(first) = members.iter().position(|(k, _)| k == key) else {
            members.push((String::from(key), val));
            return;
        };
        let mut index = 0;
        members.retain(|(k, _)| {
            index += 1;
            index - 1 == first || k != key
        });
        members[first].1 = val;
    }

    pub fn remove(&mut self, key: &str) -> bool {
        if let Value::Obj(members) = self {
            let n = members.len();
            members.retain(|(k, _)| k != key);
            return members.len() != n;
        }
        false
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Syntax {
    Tolerant,
    Commented,
    Strict,
}

pub struct Spans {
    pub at: usize,
    pub keys: Vec<usize>,
    pub inner: Vec<Spans>,
}

pub struct Marked {
    pub value: Value,
    pub spans: Spans,
    pub end: usize,
}

const BYTE_ORDER_MARK: &[u8] = &[0xEF, 0xBB, 0xBF];

pub fn parse(text: &str, syntax: Syntax) -> Option<Value> {
    let mut parser = Parser::new(text, syntax, None);
    let value = parser.root()?;
    let complete = parser.pos == text.len() && !parser.dropped_surrogate;
    (syntax != Syntax::Strict || complete).then_some(value)
}

pub fn parse_marked(text: &str, syntax: Syntax) -> Result<Marked, usize> {
    let mut parser = Parser::new(text, syntax, Some(Vec::new()));
    let Some(value) = parser.root() else {
        return Err(parser.pos);
    };
    let mut marks = parser.marks.unwrap_or_default().into_iter();
    let spans = spans_of(&value, &mut marks);
    Ok(Marked {
        value,
        spans,
        end: parser.pos,
    })
}

fn spans_of(value: &Value, marks: &mut impl Iterator<Item = usize>) -> Spans {
    let mut spans = Spans {
        at: marks.next().unwrap_or_default(),
        keys: Vec::new(),
        inner: Vec::new(),
    };
    match value {
        Value::Obj(members) => {
            for (_, member) in members {
                spans.keys.push(marks.next().unwrap_or_default());
                spans.inner.push(spans_of(member, marks));
            }
        }
        Value::Arr(items) => {
            spans.inner = items.iter().map(|item| spans_of(item, marks)).collect();
        }
        _ => {}
    }
    spans
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
    syntax: Syntax,
    depth: u32,
    dropped_surrogate: bool,
    marks: Option<Vec<usize>>,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str, syntax: Syntax, marks: Option<Vec<usize>>) -> Parser<'a> {
        Parser {
            bytes: text.as_bytes(),
            pos: 0,
            syntax,
            depth: 0,
            dropped_surrogate: false,
            marks,
        }
    }

    fn root(&mut self) -> Option<Value> {
        if self.bytes.starts_with(BYTE_ORDER_MARK) {
            self.pos = BYTE_ORDER_MARK.len();
        }
        self.ws();
        let value = self.value()?;
        self.ws();
        Some(value)
    }

    fn mark(&mut self) {
        if let Some(marks) = &mut self.marks {
            marks.push(self.pos);
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn ws(&mut self) {
        loop {
            match self.peek() {
                Some(b' ' | b'\t' | b'\n' | b'\r') => self.pos += 1,
                Some(b'/') if self.syntax == Syntax::Commented => {
                    match self.bytes.get(self.pos + 1) {
                        Some(b'/') => {
                            while let Some(c) = self.peek() {
                                if c == b'\n' {
                                    break;
                                }
                                self.pos += 1;
                            }
                        }
                        Some(b'*') => {
                            self.pos += 2;
                            while self.pos < self.bytes.len()
                                && !self.bytes[self.pos..].starts_with(b"*/")
                            {
                                self.pos += 1;
                            }
                            self.pos = (self.pos + 2).min(self.bytes.len());
                        }
                        _ => return,
                    }
                }
                _ => return,
            }
        }
    }

    fn value(&mut self) -> Option<Value> {
        self.mark();
        match self.peek()? {
            b'{' => self.object(),
            b'[' => self.array(),
            b'"' => self.string().map(Value::Str),
            b't' => self.word(b"true", Value::Bool(true)),
            b'f' => self.word(b"false", Value::Bool(false)),
            b'n' => self.word(b"null", Value::Null),
            b'-' | b'0'..=b'9' => self.number(),
            _ => None,
        }
    }

    fn word(&mut self, w: &[u8], v: Value) -> Option<Value> {
        if self.bytes[self.pos..].starts_with(w) {
            self.pos += w.len();
            Some(v)
        } else {
            None
        }
    }

    fn enter(&mut self) -> Option<()> {
        self.depth += 1;
        if self.depth > 200 {
            None
        } else {
            Some(())
        }
    }

    fn skip_commas(&mut self) {
        loop {
            self.ws();
            if self.peek() != Some(b',') {
                return;
            }
            self.pos += 1;
        }
    }

    fn item_follows(&mut self, close: u8, first: bool) -> Option<bool> {
        if self.syntax != Syntax::Strict {
            self.skip_commas();
            return Some(self.peek()? != close);
        }
        self.ws();
        if self.peek()? == close {
            return Some(false);
        }
        if !first {
            if self.peek()? != b',' {
                return None;
            }
            self.pos += 1;
            self.ws();
        }
        (self.peek()? != close).then_some(true)
    }

    fn object(&mut self) -> Option<Value> {
        self.enter()?;
        self.pos += 1;
        let mut members = Vec::new();
        loop {
            if !self.item_follows(b'}', members.is_empty())? {
                break;
            }
            if self.peek()? != b'"' {
                return None;
            }
            self.mark();
            let k = self.string()?;
            self.ws();
            if self.peek()? != b':' {
                return None;
            }
            self.pos += 1;
            self.ws();
            let v = self.value()?;
            members.push((k, v));
        }
        self.pos += 1;
        self.depth -= 1;
        Some(Value::Obj(members))
    }

    fn array(&mut self) -> Option<Value> {
        self.enter()?;
        self.pos += 1;
        let mut items = Vec::new();
        loop {
            if !self.item_follows(b']', items.is_empty())? {
                break;
            }
            items.push(self.value()?);
        }
        self.pos += 1;
        self.depth -= 1;
        Some(Value::Arr(items))
    }

    fn hex4(&mut self) -> Option<u32> {
        let s = self.bytes.get(self.pos..self.pos + 4)?;
        let mut n = 0u32;
        for &c in s {
            n = n * 16 + (c as char).to_digit(16)?;
        }
        self.pos += 4;
        Some(n)
    }

    fn string(&mut self) -> Option<String> {
        self.pos += 1;
        let mut out: Vec<u8> = Vec::new();
        loop {
            let c = self.peek()?;
            self.pos += 1;
            match c {
                b'"' => break,
                b'\\' => {
                    let e = self.peek()?;
                    self.pos += 1;
                    match e {
                        b'"' => out.push(b'"'),
                        b'\\' => out.push(b'\\'),
                        b'/' => out.push(b'/'),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'u' => {
                            let mut cp = self.hex4()?;
                            if (0xD800..0xDC00).contains(&cp) {
                                if self.bytes[self.pos..].starts_with(b"\\u") {
                                    let save = self.pos;
                                    self.pos += 2;
                                    let lo = self.hex4()?;
                                    if (0xDC00..0xE000).contains(&lo) {
                                        cp = 0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00);
                                    } else {
                                        self.pos = save;
                                        self.dropped_surrogate = true;
                                        continue;
                                    }
                                } else {
                                    self.dropped_surrogate = true;
                                    continue;
                                }
                            }
                            if let Some(ch) = char::from_u32(cp) {
                                let mut buf = [0u8; 4];
                                out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                            } else {
                                self.dropped_surrogate = true;
                            }
                        }
                        _ => return None,
                    }
                }
                _ => out.push(c),
            }
        }
        String::from_utf8(out).ok()
    }

    fn number(&mut self) -> Option<Value> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        match self.peek()? {
            b'0' => self.pos += 1,
            b'1'..=b'9' => self.digits(),
            _ => return None,
        }
        if self.peek() == Some(b'.') {
            self.pos += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return None;
            }
            self.digits();
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return None;
            }
            self.digits();
        }
        let raw = core::str::from_utf8(&self.bytes[start..self.pos]).ok()?;
        let n: f64 = raw.parse().ok().filter(|n: &f64| n.is_finite())?;
        Some(Value::Num(n, String::from(raw)))
    }

    fn digits(&mut self) {
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
    }
}

pub fn pretty(v: &Value) -> String {
    let mut out = String::new();
    write_value(v, 0, &mut out);
    out.push('\n');
    out
}

fn indent(n: usize, out: &mut String) {
    for _ in 0..n {
        out.push(' ');
    }
}

fn write_value(value: &Value, ind: usize, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Num(_, raw) => out.push_str(raw),
        Value::Str(s) => quote(s, out),
        Value::Arr(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push_str("[\n");
            for (i, x) in items.iter().enumerate() {
                indent(ind + 2, out);
                write_value(x, ind + 2, out);
                if i + 1 < items.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            indent(ind, out);
            out.push(']');
        }
        Value::Obj(members) => {
            let mut keys: Vec<&String> = Vec::new();
            for (k, _) in members {
                if !keys.contains(&k) {
                    keys.push(k);
                }
            }
            if keys.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push_str("{\n");
            for (i, k) in keys.iter().enumerate() {
                indent(ind + 2, out);
                quote(k, out);
                out.push_str(": ");
                write_value(value.get(k).unwrap_or(&Value::Null), ind + 2, out);
                if i + 1 < keys.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            indent(ind, out);
            out.push('}');
        }
    }
}

fn quote(s: &str, out: &mut String) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str("\\u00");
                out.push(HEX[(c as usize) >> 4] as char);
                out.push(HEX[(c as usize) & 15] as char);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn out_of_range_numbers_are_refused() {
        assert!(parse("{\"a\": 1e999}", Syntax::Tolerant).is_none());
        assert!(parse("[-1e999]", Syntax::Tolerant).is_none());
        let tiny = parse("[1e-999]", Syntax::Tolerant).unwrap();
        assert_eq!(tiny.arr().unwrap()[0].num(), Some(0.0));
    }

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
}
