use super::args::split_command_line;
use alloc::string::String;
use alloc::vec::Vec;

fn split(line: &str) -> Vec<String> {
    let units: Vec<u16> = line.encode_utf16().collect();
    split_command_line(&units)
}

#[test]
fn the_program_name_drops_its_quotes_and_keeps_its_backslashes() {
    let cases: [(&str, &[&str]); 3] = [
        (
            r#""C:\Program Files\statusline.exe" --bench 10"#,
            &[r"C:\Program Files\statusline.exe", "--bench", "10"],
        ),
        (
            r#"C:\bin\"st atus"\line.exe x"#,
            &[r"C:\bin\st atus\line.exe", "x"],
        ),
        ("", &[""]),
    ];
    for (line, expected) in cases {
        assert_eq!(split(line), expected, "{}", line);
    }
}

#[test]
fn arguments_follow_the_documented_quote_and_backslash_rules() {
    let cases: [(&str, &[&str]); 6] = [
        (r#"x "abc" d e"#, &["x", "abc", "d", "e"]),
        (r#"x a\\b d"e f"g h"#, &["x", r"a\\b", "de fg", "h"]),
        (r#"x a\\\"b c d"#, &["x", r#"a\"b"#, "c", "d"]),
        (r#"x a\\\\"b c" d e"#, &["x", r"a\\b c", "d", "e"]),
        (r#"x a"b"" c d"#, &["x", r#"ab" c d"#]),
        (r#"x "ab\"c" "\\" d"#, &["x", r#"ab"c"#, r"\", "d"]),
    ];
    for (line, expected) in cases {
        assert_eq!(split(line), expected, "{}", line);
    }
}

#[test]
fn empty_arguments_stay_and_extra_blanks_go() {
    let cases: [(&str, &[&str]); 4] = [
        ("x \"\" a\t\t\"\"  ", &["x", "", "a", ""]),
        ("x \"\"", &["x", ""]),
        ("x \"a b", &["x", "a b"]),
        (" a  b", &["", "a", "b"]),
    ];
    for (line, expected) in cases {
        assert_eq!(split(line), expected, "{}", line);
    }
}
