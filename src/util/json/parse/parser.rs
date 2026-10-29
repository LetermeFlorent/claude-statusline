use super::Syntax;
use crate::json::Value;
use alloc::vec::Vec;

const BYTE_ORDER_MARK: &[u8] = &[0xEF, 0xBB, 0xBF];

pub(super) struct Parser<'a> {
    pub(super) bytes: &'a [u8],
    pub(super) pos: usize,
    pub(super) syntax: Syntax,
    pub(super) depth: u32,
    pub(super) dropped_surrogate: bool,
    pub(super) marks: Option<Vec<usize>>,
}

impl<'a> Parser<'a> {
    pub(super) fn new(text: &'a str, syntax: Syntax, marks: Option<Vec<usize>>) -> Parser<'a> {
        Parser {
            bytes: text.as_bytes(),
            pos: 0,
            syntax,
            depth: 0,
            dropped_surrogate: false,
            marks,
        }
    }

    pub(super) fn root(&mut self) -> Option<Value> {
        if self.bytes.starts_with(BYTE_ORDER_MARK) {
            self.pos = BYTE_ORDER_MARK.len();
        }
        self.ws();
        let value = self.value()?;
        self.ws();
        Some(value)
    }

    pub(super) fn mark(&mut self) {
        if let Some(marks) = &mut self.marks {
            marks.push(self.pos);
        }
    }

    pub(super) fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    pub(super) fn value(&mut self) -> Option<Value> {
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

    pub(super) fn enter(&mut self) -> Option<()> {
        self.depth += 1;
        if self.depth > 200 {
            None
        } else {
            Some(())
        }
    }
}
