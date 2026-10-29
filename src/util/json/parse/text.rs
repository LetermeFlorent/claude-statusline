use super::parser::Parser;
use alloc::string::String;
use alloc::vec::Vec;

impl Parser<'_> {
    fn hex4(&mut self) -> Option<u32> {
        let s = self.bytes.get(self.pos..self.pos + 4)?;
        let mut n = 0u32;
        for &c in s {
            n = n * 16 + (c as char).to_digit(16)?;
        }
        self.pos += 4;
        Some(n)
    }

    fn low_surrogate(&mut self, high: u32) -> Option<Option<u32>> {
        if !self.bytes[self.pos..].starts_with(b"\\u") {
            return Some(None);
        }
        let save = self.pos;
        self.pos += 2;
        let low = self.hex4()?;
        if !(0xDC00..0xE000).contains(&low) {
            self.pos = save;
            return Some(None);
        }
        Some(Some(0x10000 + ((high - 0xD800) << 10) + (low - 0xDC00)))
    }

    fn unicode(&mut self, out: &mut Vec<u8>) -> Option<()> {
        let mut cp = self.hex4()?;
        if (0xD800..0xDC00).contains(&cp) {
            match self.low_surrogate(cp)? {
                Some(pair) => cp = pair,
                None => {
                    self.dropped_surrogate = true;
                    return Some(());
                }
            }
        }
        match char::from_u32(cp) {
            Some(ch) => {
                let mut buf = [0u8; 4];
                out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
            }
            None => self.dropped_surrogate = true,
        }
        Some(())
    }

    fn escape(&mut self, out: &mut Vec<u8>) -> Option<()> {
        let e = self.peek()?;
        self.pos += 1;
        let byte = match e {
            b'"' | b'\\' | b'/' => e,
            b'b' => 8,
            b'f' => 12,
            b'n' => b'\n',
            b'r' => b'\r',
            b't' => b'\t',
            b'u' => return self.unicode(out),
            _ => return None,
        };
        out.push(byte);
        Some(())
    }

    pub(super) fn string(&mut self) -> Option<String> {
        self.pos += 1;
        let mut out: Vec<u8> = Vec::new();
        loop {
            let c = self.peek()?;
            self.pos += 1;
            match c {
                b'"' => break,
                b'\\' => self.escape(&mut out)?,
                _ => out.push(c),
            }
        }
        String::from_utf8(out).ok()
    }
}
