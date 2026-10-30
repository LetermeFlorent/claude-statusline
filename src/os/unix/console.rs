use std::io::{IsTerminal, Read, Write};

pub fn stdin() -> Option<String> {
    let mut input = std::io::stdin();
    if input.is_terminal() {
        return None;
    }
    let mut data = Vec::new();
    let _ = input.read_to_end(&mut data);
    Some(String::from_utf8_lossy(&data).into_owned())
}

pub fn out(s: &str) {
    let mut o = std::io::stdout().lock();
    let _ = o.write_all(s.as_bytes());
    let _ = o.flush();
}

pub fn err(s: &str) {
    let _ = std::io::stderr().write_all(s.as_bytes());
}
