use super::{percent, SESSIONS};
use crate::common::Home;
use std::process::Output;

fn five_hour_percent(output: &Output) -> Option<String> {
    let text = String::from_utf8_lossy(&output.stdout);
    let segment = text.split(" | ").find(|part| part.starts_with("5h "))?;
    segment
        .split(' ')
        .find(|word| word.ends_with('%'))
        .map(String::from)
}

fn whole_bar(shown: &str) -> bool {
    let lines: Vec<&str> = shown.split_terminator('\n').collect();
    shown.ends_with('\n')
        && lines.len() == 2
        && lines[0].starts_with("ctx ")
        && lines[0].ends_with(" | Opus")
        && lines[1].starts_with("ram ")
        && lines[1].matches(" | ").count() == 2
}

pub fn assert_shows(outputs: &[Output], round: usize) {
    for (account, output) in outputs.iter().enumerate() {
        let shown = String::from_utf8_lossy(&output.stdout);
        let errors = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "account {}: {}", account, errors);
        assert!(errors.is_empty(), "account {}: {}", account, errors);
        assert!(whole_bar(&shown), "account {}: {:?}", account, shown);
        let expected = format!("{}%", percent(account, round));
        assert_eq!(
            five_hour_percent(output).as_deref(),
            Some(expected.as_str()),
            "account {}: {}",
            account,
            shown
        );
    }
}

pub fn assert_stored(home: &Home, round: usize) -> String {
    let state = std::fs::read_to_string(home.state_dir().join("state")).unwrap();
    assert!(
        state
            .lines()
            .all(|line| line.split_once(' ').is_some_and(|(key, _)| !key.is_empty())),
        "{}",
        state
    );
    for account in 0..SESSIONS {
        let line = format!("q5_pct@claude-a{} {}", account, percent(account, round));
        assert!(
            state.lines().any(|kept| kept == line),
            "{} lost:\n{}",
            line,
            state
        );
    }
    let leftovers: Vec<String> = std::fs::read_dir(home.state_dir())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".tmp") || name.ends_with(".changes"))
        .collect();
    assert!(leftovers.is_empty(), "{:?}", leftovers);
    state
}
