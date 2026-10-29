use super::parser::Parser;
use super::Syntax;
use crate::json::Value;
use alloc::vec::Vec;

impl Parser<'_> {
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

    pub(super) fn object(&mut self) -> Option<Value> {
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

    pub(super) fn array(&mut self) -> Option<Value> {
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
}
