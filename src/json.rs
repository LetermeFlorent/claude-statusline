use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    // Le texte d'origine est garde pour reecrire un nombre tel que l'utilisateur l'a saisi
    Num(f64, String),
    Str(String),
    Arr(Vec<Value>),
    Obj(Vec<(String, Value)>),
}

impl Value {
    // Cle en double : la derniere gagne, comme dans l'ancien binaire
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
        if let Value::Obj(m) = self {
            m.retain(|(k, _)| k != key);
            m.push((String::from(key), val));
        }
    }

    pub fn remove(&mut self, key: &str) -> bool {
        if let Value::Obj(m) = self {
            let n = m.len();
            m.retain(|(k, _)| k != key);
            return m.len() != n;
        }
        false
    }
}

// lenient : commentaires // et /* */ (fichiers de reglages).
// Le texte qui suit la premiere valeur est ignore.
pub fn parse(text: &str, lenient: bool) -> Option<Value> {
    let b = text.as_bytes();
    let mut p = Parser { b, i: 0, lenient, depth: 0 };
    if b.starts_with(&[0xEF, 0xBB, 0xBF]) {
        p.i = 3;
    }
    p.ws();
    p.value()
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
    lenient: bool,
    depth: u32,
}

impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }

    fn ws(&mut self) {
        loop {
            match self.peek() {
                Some(b' ' | b'\t' | b'\n' | b'\r') => self.i += 1,
                Some(b'/') if self.lenient => match self.b.get(self.i + 1) {
                    Some(b'/') => {
                        while let Some(c) = self.peek() {
                            if c == b'\n' {
                                break;
                            }
                            self.i += 1;
                        }
                    }
                    Some(b'*') => {
                        self.i += 2;
                        while self.i < self.b.len() && !self.b[self.i..].starts_with(b"*/") {
                            self.i += 1;
                        }
                        self.i = (self.i + 2).min(self.b.len());
                    }
                    _ => return,
                },
                _ => return,
            }
        }
    }

    fn value(&mut self) -> Option<Value> {
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
        if self.b[self.i..].starts_with(w) {
            self.i += w.len();
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

    // Les virgules entre elements sont facultatives et peuvent se repeter,
    // comme dans l'ancien binaire
    fn seps(&mut self) {
        loop {
            self.ws();
            if self.peek() != Some(b',') {
                return;
            }
            self.i += 1;
        }
    }

    fn object(&mut self) -> Option<Value> {
        self.enter()?;
        self.i += 1;
        let mut m = Vec::new();
        loop {
            self.seps();
            match self.peek()? {
                b'}' => break,
                b'"' => {}
                _ => return None,
            }
            let k = self.string()?;
            self.ws();
            if self.peek()? != b':' {
                return None;
            }
            self.i += 1;
            self.ws();
            let v = self.value()?;
            m.push((k, v));
        }
        self.i += 1;
        self.depth -= 1;
        Some(Value::Obj(m))
    }

    fn array(&mut self) -> Option<Value> {
        self.enter()?;
        self.i += 1;
        let mut a = Vec::new();
        loop {
            self.seps();
            if self.peek()? == b']' {
                break;
            }
            a.push(self.value()?);
        }
        self.i += 1;
        self.depth -= 1;
        Some(Value::Arr(a))
    }

    fn hex4(&mut self) -> Option<u32> {
        let s = self.b.get(self.i..self.i + 4)?;
        let mut n = 0u32;
        for &c in s {
            n = n * 16 + (c as char).to_digit(16)?;
        }
        self.i += 4;
        Some(n)
    }

    fn string(&mut self) -> Option<String> {
        self.i += 1;
        let mut out: Vec<u8> = Vec::new();
        loop {
            let c = self.peek()?;
            self.i += 1;
            match c {
                b'"' => break,
                b'\\' => {
                    let e = self.peek()?;
                    self.i += 1;
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
                                if self.b[self.i..].starts_with(b"\\u") {
                                    let save = self.i;
                                    self.i += 2;
                                    let lo = self.hex4()?;
                                    if (0xDC00..0xE000).contains(&lo) {
                                        cp = 0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00);
                                    } else {
                                        self.i = save;
                                        continue;
                                    }
                                } else {
                                    continue;
                                }
                            }
                            // Demi-paire orpheline : abandonnee, comme avant
                            if let Some(ch) = char::from_u32(cp) {
                                let mut buf = [0u8; 4];
                                out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
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
        let start = self.i;
        if self.peek() == Some(b'-') {
            self.i += 1;
        }
        match self.peek()? {
            b'0' => self.i += 1,
            b'1'..=b'9' => self.digits(),
            _ => return None,
        }
        if self.peek() == Some(b'.') {
            self.i += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return None;
            }
            self.digits();
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.i += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.i += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return None;
            }
            self.digits();
        }
        let raw = core::str::from_utf8(&self.b[start..self.i]).ok()?;
        let n: f64 = raw.parse().ok()?;
        Some(Value::Num(n, String::from(raw)))
    }

    fn digits(&mut self) {
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.i += 1;
        }
    }
}

// Ecriture indentee de 2 espaces, cles triees, comme serde_json avec une BTreeMap
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

fn write_value(v: &Value, ind: usize, out: &mut String) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Num(_, raw) => out.push_str(raw),
        Value::Str(s) => quote(s, out),
        Value::Arr(a) => {
            if a.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push_str("[\n");
            for (i, x) in a.iter().enumerate() {
                indent(ind + 2, out);
                write_value(x, ind + 2, out);
                if i + 1 < a.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            indent(ind, out);
            out.push(']');
        }
        Value::Obj(m) => {
            let mut keys: Vec<&String> = Vec::new();
            for (k, _) in m {
                if !keys.contains(&k) {
                    keys.push(k);
                }
            }
            if keys.is_empty() {
                out.push_str("{}");
                return;
            }
            keys.sort();
            out.push_str("{\n");
            for (i, k) in keys.iter().enumerate() {
                indent(ind + 2, out);
                quote(k, out);
                out.push_str(": ");
                write_value(v.get(k).unwrap_or(&Value::Null), ind + 2, out);
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
    fn lecture() {
        let v = parse("{\"a\":1,\"a\":2,\"s\":\"\\u00e9\\ud83d\\ude00\"}", false).unwrap();
        assert_eq!(v.get("a").unwrap().num(), Some(2.0));
        assert_eq!(v.get("s").unwrap().str(), Some("\u{e9}\u{1f600}"));
        assert!(parse("{,\"a\":1,,}", false).is_some());
        assert!(parse("[1,,2,]", false).unwrap().arr().unwrap().len() == 2);
        assert!(parse("{\"a\" 1}", false).is_none());
        assert!(parse("{\"a\":1, // x\n}", true).is_some());
        assert!(parse("\u{feff}{\"a\":1}", false).is_some());
        assert_eq!(parse("{\"s\":\"a\\ud83db\"}", false).unwrap().get("s").unwrap().str(), Some("ab"));
        assert!(parse("[1.]", false).is_none());
        assert_eq!(parse("{\"x\":1} reste", false).unwrap().get("x").unwrap().num(), Some(1.0));
    }

    #[test]
    fn ecriture() {
        let v = parse("{\"b\":[1,2.50],\"a\":{},\"c\":\"x\\u0001\\\"\",\"d\":[]}", false).unwrap();
        assert_eq!(pretty(&v), "{\n  \"a\": {},\n  \"b\": [\n    1,\n    2.50\n  ],\n  \"c\": \"x\\u0001\\\"\",\n  \"d\": []\n}\n");
    }
}
