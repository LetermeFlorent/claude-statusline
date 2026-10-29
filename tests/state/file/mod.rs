mod quotas;
mod saving;

use std::io::Write;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const ACCOUNTS: [(&str, &str); 3] = [
    (".claude", "claude"),
    (".claude-compte2", "claude-compte2"),
    (".claude-compte3", "claude-compte3"),
];
const FIVE_HOURS: u64 = 5 * 3600;
const SEVEN_DAYS: u64 = 7 * 24 * 3600;

type Window = (u64, u64);

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn input(five: Window, seven: Window) -> String {
    format!(
        "{{\"model\":{{\"display_name\":\"Opus\"}},\"rate_limits\":{{\"five_hour\":{{\"used_percentage\":{},\"resets_at\":{}}},\"seven_day\":{{\"used_percentage\":{},\"resets_at\":{}}}}}}}",
        five.0, five.1, seven.0, seven.1
    )
}

fn limits(account: usize, start: u64) -> (Window, Window) {
    let five = (21 + account as u64, start + FIVE_HOURS);
    let seven = (41 + account as u64, start + SEVEN_DAYS);
    (five, seven)
}

fn render(mut command: Command, input: &str) {
    let mut child = command.spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(input.as_bytes()).unwrap();
    drop(stdin);
    let output = child.wait_with_output().unwrap();
    let errors = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success() && errors.is_empty(), "{}", errors);
}

fn lines_of<'a>(state: &'a str, tag: &str) -> Vec<&'a str> {
    let suffix = format!("@{}", tag);
    let mut lines: Vec<&str> = state
        .lines()
        .filter(|line| {
            line.split_once(' ')
                .is_some_and(|(key, _)| key.ends_with(&suffix))
        })
        .collect();
    lines.sort();
    lines
}
