use crate::json::{self, Syntax};
use alloc::format;
use alloc::string::String;

const USER: &[u8] = b"\"type\":\"user\"";
const USAGE: &[u8] = b"\"usage\":{";
const HEAD_BYTES: usize = 300;
/// How long an arrow stays lit after its count went up.
const RISE_SECONDS: u64 = 8;

fn has(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}

/// Running token totals of one subagent, read from its transcript and resumed from `offset`.
/// A message is written once per content block with the same usage, so only the last usage
/// seen for a message id counts, and it is added to the totals when the next id shows up.
#[derive(Debug, Default, PartialEq)]
pub struct Usage {
    pub offset: u64,
    sent: u64,
    got: u64,
    last_sent: u64,
    last_got: u64,
    rose_sent: u64,
    rose_got: u64,
    id: String,
}

impl Usage {
    /// Reads the complete lines of `chunk` and returns how many bytes they take.
    pub fn advance(&mut self, chunk: &[u8]) -> usize {
        let end = chunk.iter().rposition(|b| *b == b'\n').map_or(0, |i| i + 1);
        chunk[..end]
            .split(|b| *b == b'\n')
            .for_each(|l| self.line(l));
        end
    }

    fn line(&mut self, line: &[u8]) {
        if !has(line, USAGE) || has(&line[..line.len().min(HEAD_BYTES)], USER) {
            return;
        }
        let Some(v) = core::str::from_utf8(line)
            .ok()
            .and_then(|t| json::parse(t, Syntax::Tolerant))
        else {
            return;
        };
        let (Some(m), Some("assistant")) = (v.get("message"), v.get("type").and_then(|t| t.str()))
        else {
            return;
        };
        let (Some(id), Some(u)) = (m.get("id").and_then(|i| i.str()), m.get("usage")) else {
            return;
        };
        let n = |k: &str| u.get(k).and_then(|x| x.num()).map_or(0, |n| n as u64);
        if self.id != id {
            self.sent = self.sent.saturating_add(self.last_sent);
            self.got = self.got.saturating_add(self.last_got);
            self.id = String::from(id);
        }
        // shortcut: cache reads are not counted as sent, they re-read the same context each turn
        self.last_sent = n("input_tokens").saturating_add(n("cache_creation_input_tokens"));
        self.last_got = n("output_tokens");
    }

    pub fn totals(&self) -> (u64, u64) {
        (
            self.sent.saturating_add(self.last_sent),
            self.got.saturating_add(self.last_got),
        )
    }

    /// Notes, at time `now` in seconds, which total went up since `before`.
    pub fn note_rise(&mut self, before: (u64, u64), now: u64) {
        let (sent, got) = self.totals();
        if sent > before.0 {
            self.rose_sent = now;
        }
        if got > before.1 {
            self.rose_got = now;
        }
    }

    pub fn used(&self, now: u64) -> super::Used {
        let (sent, got) = self.totals();
        let recent = |at: u64| at > 0 && now.saturating_sub(at) <= RISE_SECONDS;
        super::Used {
            sent,
            got,
            sent_rising: recent(self.rose_sent),
            got_rising: recent(self.rose_got),
        }
    }

    pub fn encode(&self) -> String {
        format!(
            "{} {} {} {} {} {} {} {}",
            self.offset,
            self.sent,
            self.got,
            self.last_sent,
            self.last_got,
            self.rose_sent,
            self.rose_got,
            self.id
        )
    }

    pub fn decode(text: &str) -> Option<Self> {
        let mut p = text.split_whitespace();
        let mut num = || p.next()?.parse().ok();
        Some(Self {
            offset: num()?,
            sent: num()?,
            got: num()?,
            last_sent: num()?,
            last_got: num()?,
            rose_sent: num()?,
            rose_got: num()?,
            id: String::from(p.next().unwrap_or("")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &str = r#"{"agentId":"a","message":{"id":"m1","role":"assistant","usage":{"input_tokens":2,"cache_creation_input_tokens":8,"cache_read_input_tokens":500,"output_tokens":30}},"type":"assistant"}"#;
    const B: &str = r#"{"agentId":"a","message":{"id":"m2","role":"assistant","usage":{"input_tokens":1,"cache_creation_input_tokens":4,"output_tokens":5}},"type":"assistant"}"#;
    const U: &str = r#"{"type":"user","message":{"role":"user","content":"x \"usage\":{"}}"#;

    #[test]
    fn a_message_repeated_per_block_counts_once() {
        let mut u = Usage::default();
        let text = format!("{A}\n{U}\n{A}\n{B}\n");
        assert_eq!(u.advance(text.as_bytes()), text.len());
        assert_eq!(u.totals(), (15, 35));
    }

    #[test]
    fn a_line_still_being_written_waits() {
        let mut u = Usage::default();
        let text = format!("{A}\n{}", &B[..40]);
        assert_eq!(u.advance(text.as_bytes()), A.len() + 1);
        assert_eq!(u.totals(), (10, 30));
    }

    #[test]
    fn a_total_that_went_up_lights_its_arrow_for_a_few_seconds() {
        let mut u = Usage::default();
        u.advance(format!("{A}\n").as_bytes());
        u.note_rise((10, 30), 100);
        u.advance(format!("{B}\n").as_bytes());
        u.note_rise((10, 30), 100);
        assert_eq!(
            (u.used(100).sent_rising, u.used(100).got_rising),
            (true, true)
        );
        assert!(u.used(108).sent_rising);
        assert!(!u.used(109).sent_rising);
        assert!(!Usage::default().used(5).got_rising);
    }

    #[test]
    fn the_cache_text_round_trips() {
        let mut u = Usage::default();
        u.advance(format!("{A}\n{B}\n").as_bytes());
        u.offset = 99;
        u.note_rise((0, 0), 7);
        assert_eq!(Usage::decode(&u.encode()), Some(u));
        assert_eq!(Usage::decode("1 2"), None);
    }
}
