use super::parser::Parser;
use super::Syntax;

impl Parser<'_> {
    pub(super) fn ws(&mut self) {
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
}
