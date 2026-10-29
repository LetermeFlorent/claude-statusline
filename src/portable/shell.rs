use super::path::{home_rest, slashes, tilde_is_home};
use alloc::format;
use alloc::string::String;
use core::str::Chars;

const SHELL_SAFE_PUNCTUATION: &str = "_./:@%+=,-";
const DOUBLE_QUOTE_ESCAPES: &str = "$`\"\\";

fn shell_safe(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || SHELL_SAFE_PUNCTUATION.contains(c))
}

pub(super) fn quoted(text: &str) -> String {
    if shell_safe(text) {
        return String::from(text);
    }
    format!("'{}'", text.replace('\'', r"'\''"))
}

fn bare(c: char) -> bool {
    !c.is_ascii() || c.is_ascii_alphanumeric() || c == '~' || SHELL_SAFE_PUNCTUATION.contains(c)
}

fn single_quoted(chars: &mut Chars, word: &mut String) -> Option<()> {
    for c in chars.by_ref() {
        if c == '\'' {
            return Some(());
        }
        word.push(c);
    }
    None
}

fn double_quoted(chars: &mut Chars, word: &mut String) -> Option<()> {
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(()),
            '$' | '`' => return None,
            '\\' => {
                let escaped = chars.next()?;
                if !DOUBLE_QUOTE_ESCAPES.contains(escaped) {
                    word.push('\\');
                }
                word.push(escaped);
            }
            c => word.push(c),
        }
    }
    None
}

pub fn unquoted(command: &str) -> Option<String> {
    let mut word = String::new();
    let mut chars = command.trim().chars();
    while let Some(c) = chars.next() {
        match c {
            '\'' => single_quoted(&mut chars, &mut word)?,
            '"' => double_quoted(&mut chars, &mut word)?,
            '\\' => word.push(chars.next()?),
            c if bare(c) => word.push(c),
            _ => return None,
        }
    }
    (!word.is_empty()).then_some(word)
}

pub fn command(exe: &str) -> String {
    match home_rest(exe).filter(|_| tilde_is_home()) {
        Some(rest) if shell_safe(&rest) => format!("~/{}", rest),
        _ => quoted(&slashes(exe)),
    }
}
