use super::schema::{Keys, Kind, SETTINGS};
use crate::json::{self, Spans, Syntax, Value};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, PartialEq)]
pub enum Fault {
    InvalidJson,
    NotAnObject,
    TrailingText,
    UnknownKey(String),
    Expects(String, &'static Kind),
}

pub struct Problem {
    pub at: usize,
    pub fault: Fault,
}

#[derive(Clone, Copy)]
struct Node<'a> {
    value: &'a Value,
    spans: &'a Spans,
}

pub fn problems(text: &str) -> Vec<Problem> {
    let marked = match json::parse_marked(text, Syntax::Commented) {
        Ok(marked) => marked,
        Err(at) => {
            let fault = Fault::InvalidJson;
            return alloc::vec![Problem { at, fault }];
        }
    };
    let mut found = Vec::new();
    let root = Node {
        value: &marked.value,
        spans: &marked.spans,
    };
    if marked.value.is_obj() {
        members(SETTINGS, "", root, &mut found);
    } else {
        let fault = Fault::NotAnObject;
        found.push(Problem {
            at: root.spans.at,
            fault,
        });
    }
    if marked.end < text.len() {
        let fault = Fault::TrailingText;
        found.push(Problem {
            at: marked.end,
            fault,
        });
    }
    found
}

fn members(keys: Keys, path: &str, node: Node, found: &mut Vec<Problem>) {
    let Value::Obj(members) = node.value else {
        return;
    };
    let entries = members.iter().zip(&node.spans.keys).zip(&node.spans.inner);
    for (((key, value), &key_at), spans) in entries {
        let path = if path.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", path, key)
        };
        match keys.iter().find(|(name, _)| *name == key.as_str()) {
            Some((_, kind)) => check(kind, &path, Node { value, spans }, found),
            None => {
                let fault = Fault::UnknownKey(path);
                found.push(Problem { at: key_at, fault });
            }
        }
    }
}

fn check(kind: &'static Kind, path: &str, node: Node, found: &mut Vec<Problem>) {
    if matches!(node.value, Value::Null) {
        return;
    }
    if !kind.accepts(node.value) {
        let fault = Fault::Expects(String::from(path), kind);
        found.push(Problem {
            at: node.spans.at,
            fault,
        });
        return;
    }
    match (kind, node.value) {
        (Kind::Object(keys), _) => members(keys, path, node, found),
        (Kind::List(item), Value::Arr(items)) => {
            for (value, spans) in items.iter().zip(&node.spans.inner) {
                check(item, path, Node { value, spans }, found);
            }
        }
        _ => {}
    }
}
