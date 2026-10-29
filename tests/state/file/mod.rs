mod common;
mod state_folder;

use common::Home;
use std::io::{Read, Write};
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

#[test]
fn each_account_keeps_its_quotas_under_the_tag_of_its_folder() {
    let home = Home::new("state-tags");
    let start = now();
    for (account, (folder, _)) in ACCOUNTS.iter().enumerate() {
        let (five, seven) = limits(account, start);
        render(home.session_in(folder), &input(five, seven));
    }
    let end = now();
    let state = std::fs::read_to_string(home.state_dir().join("state")).unwrap();
    for (account, (_, tag)) in ACCOUNTS.iter().enumerate() {
        let ((five_pct, five_at), (seven_pct, seven_at)) = limits(account, start);
        let lines = lines_of(&state, tag);
        let stamp_key = format!("q_at@{} ", tag);
        let stamp = lines
            .iter()
            .find_map(|line| line.strip_prefix(&stamp_key)?.parse::<u64>().ok());
        assert!(
            stamp.is_some_and(|at| (start..=end).contains(&at)),
            "{}",
            state
        );
        let mut expected = vec![
            format!("q5_at@{} {}", tag, five_at),
            format!("q5_pct@{} {}", tag, five_pct),
            format!("q7_at@{} {}", tag, seven_at),
            format!("q7_pct@{} {}", tag, seven_pct),
            format!("{}{}", stamp_key, stamp.unwrap()),
        ];
        expected.sort();
        assert_eq!(lines, expected, "{}", state);
    }
}

#[test]
fn a_reader_holding_the_file_keeps_its_copy_while_the_next_render_saves() {
    let home = Home::new("state-reader");
    let start = now();
    let (five, seven) = limits(0, start);
    render(home.session_in(".claude"), &input(five, seven));
    let path = home.state_dir().join("state");
    let mut reader = std::fs::File::open(&path).unwrap();
    let (five, seven) = limits(1, start);
    render(home.session_in(".claude"), &input(five, seven));
    let mut held = String::new();
    reader.read_to_string(&mut held).unwrap();
    assert!(
        held.lines().any(|line| line == "q5_pct@claude 21"),
        "{}",
        held
    );
    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(
        saved.lines().any(|line| line == "q5_pct@claude 22"),
        "{}",
        saved
    );
}

#[test]
fn saving_leaves_only_the_state_and_its_lock_beside_it() {
    let home = Home::new("state-files");
    let start = now();
    for account in 0..ACCOUNTS.len() {
        let (five, seven) = limits(account, start);
        render(home.session(account), &input(five, seven));
    }
    let mut names: Vec<String> = std::fs::read_dir(home.state_dir())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["state", "state.lock"]);
}

#[test]
fn projects_alternating_in_vscode_keep_one_theme_entry_each() {
    let home = Home::new("theme-slots");
    let folders = ["one", "two"].map(|name| {
        let folder = home.0.join(name);
        std::fs::create_dir_all(folder.join(".vscode")).unwrap();
        let settings = format!("{{\"workbench.colorTheme\": \"{}\"}}", name);
        std::fs::write(folder.join(".vscode").join("settings.json"), settings).unwrap();
        folder.to_string_lossy().replace('\\', "\\\\")
    });
    for folder in folders.iter().cycle().take(4) {
        let mut command = home.session(0);
        command.env("TERM_PROGRAM", "vscode");
        render(command, &format!("{{\"cwd\":\"{}\"}}", folder));
    }
    let state = std::fs::read_to_string(home.state_dir().join("state")).unwrap();
    let keys: Vec<&str> = state
        .lines()
        .filter_map(|l| l.split_once(' '))
        .map(|l| l.0)
        .collect();
    assert_eq!(
        keys.iter().filter(|k| k.starts_with("bg@")).count(),
        2,
        "{}",
        state
    );
    assert!(!keys.iter().any(|k| k.starts_with("bg_")), "{}", state);
}
