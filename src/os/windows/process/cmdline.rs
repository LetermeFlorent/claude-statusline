use crate::os::windows::paths::wide;
use alloc::string::String;
use alloc::vec::Vec;

fn quote(arg: &str, out: &mut String) {
    if !arg.is_empty() && !arg.contains([' ', '\t', '"']) {
        out.push_str(arg);
        return;
    }
    out.push('"');
    let mut slashes = 0;
    for c in arg.chars() {
        if c == '\\' {
            slashes += 1;
            continue;
        }
        let n = if c == '"' { slashes * 2 + 1 } else { slashes };
        out.extend(core::iter::repeat_n('\\', n));
        slashes = 0;
        out.push(c);
    }
    out.extend(core::iter::repeat_n('\\', slashes * 2));
    out.push('"');
}

pub fn command_line(exe: &str, args: &[&str]) -> Vec<u16> {
    let mut line = String::new();
    quote(exe, &mut line);
    for arg in args {
        line.push(' ');
        quote(arg, &mut line);
    }
    wide(&line)
}
