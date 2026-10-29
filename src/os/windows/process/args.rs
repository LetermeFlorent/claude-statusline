use crate::os::windows::paths::units_until_nul;
use crate::os::windows::sys::*;
use alloc::string::String;
use alloc::vec::Vec;

pub fn args() -> Vec<String> {
    let line = unsafe { GetCommandLineW() };
    if line.is_null() {
        return Vec::new();
    }
    split_command_line(unsafe { units_until_nul(line) })
}

const QUOTE: u16 = b'"' as u16;
const BACKSLASH: u16 = b'\\' as u16;
const SPACE: u16 = b' ' as u16;
const TAB: u16 = b'\t' as u16;

fn is_blank(unit: u16) -> bool {
    unit == SPACE || unit == TAB
}

fn program_end(line: &[u16]) -> usize {
    let mut quoted = false;
    line.iter()
        .position(|&unit| {
            quoted ^= unit == QUOTE;
            is_blank(unit) && !quoted
        })
        .unwrap_or(line.len())
}

#[derive(Default)]
struct Words {
    found: Vec<String>,
    word: Vec<u16>,
    started: bool,
    quoted: bool,
}

impl Words {
    fn end_word(&mut self) {
        if self.started {
            self.found.push(String::from_utf16_lossy(&self.word));
        }
        self.word.clear();
        self.started = false;
    }

    fn read(&mut self, line: &[u16], at: usize) -> usize {
        let unit = line[at];
        if is_blank(unit) && !self.quoted {
            self.end_word();
            return at + 1;
        }
        self.started = true;
        match unit {
            BACKSLASH => self.read_backslashes(line, at),
            QUOTE if self.quoted && line.get(at + 1) == Some(&QUOTE) => {
                self.word.push(QUOTE);
                at + 2
            }
            QUOTE => {
                self.quoted = !self.quoted;
                at + 1
            }
            _ => {
                self.word.push(unit);
                at + 1
            }
        }
    }

    fn read_backslashes(&mut self, line: &[u16], at: usize) -> usize {
        let count = line[at..]
            .iter()
            .take_while(|&&unit| unit == BACKSLASH)
            .count();
        let after = at + count;
        if line.get(after) != Some(&QUOTE) {
            self.word.resize(self.word.len() + count, BACKSLASH);
            return after;
        }
        self.word.resize(self.word.len() + count / 2, BACKSLASH);
        if count % 2 == 0 {
            return after;
        }
        self.word.push(QUOTE);
        after + 1
    }
}

pub fn split_command_line(line: &[u16]) -> Vec<String> {
    let end = program_end(line);
    let program: Vec<u16> = line[..end]
        .iter()
        .copied()
        .filter(|&unit| unit != QUOTE)
        .collect();
    let mut words = Words::default();
    words.found.push(String::from_utf16_lossy(&program));
    let mut at = end;
    while at < line.len() {
        at = words.read(line, at);
    }
    words.end_word();
    words.found
}
