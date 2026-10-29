use crate::json::Value;
use alloc::string::String;
use alloc::vec::Vec;

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
