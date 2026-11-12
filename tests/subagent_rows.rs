mod common;

use common::Home;
use std::io::Write;
use std::process::Output;

fn run(input: &str) -> Output {
    let home = Home::new("subagent-rows");
    let mut child = home.session(0).arg("subagents").spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn plain(line: &str) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some(at) = rest.find(r"\u001b[") {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        rest = &tail[tail.find('m').map_or(tail.len(), |m| m + 1)..];
    }
    out.push_str(rest);
    out
}

#[test]
fn each_task_gets_one_json_line() {
    let output = run(r#"{"columns":120,"tasks":[
            {"id":"a1","name":"scout","model":"claude-opus-4-7","effort":"high"},
            {"id":"b2","name":"plain"}]}"#);
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2, "{}", text);
    assert!(
        lines[0].starts_with(r#"{"id":"a1","content":""#),
        "{}",
        text
    );
    let first = plain(lines[0]);
    assert!(first.contains("Opus 4.7"), "{}", first);
    assert!(first.contains("high"), "{}", first);
    assert!(
        lines[1].starts_with(r#"{"id":"b2","content":""#),
        "{}",
        text
    );
    assert!(lines[1].contains("plain"), "{}", text);
    assert!(!lines[1].contains("Opus"), "{}", text);
}

#[test]
fn invalid_input_prints_nothing() {
    let output = run("not json");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty());
}
