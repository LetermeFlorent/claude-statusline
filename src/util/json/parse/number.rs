use super::parser::Parser;
use crate::json::Value;
use alloc::string::String;

impl Parser<'_> {
    pub(super) fn number(&mut self) -> Option<Value> {
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
