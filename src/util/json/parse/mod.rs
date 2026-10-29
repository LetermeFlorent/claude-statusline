use crate::json::Value;
use alloc::vec::Vec;
use parser::Parser;

mod container;
mod number;
mod parser;
mod space;
mod text;

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
