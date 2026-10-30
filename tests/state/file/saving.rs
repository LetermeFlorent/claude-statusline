use super::{input, limits, lines_of, now, render, ACCOUNTS};
use crate::common::Home;
use std::time::{Duration, SystemTime};

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

#[test]
fn changes_parked_by_clm_are_merged_in_the_order_they_were_written() {
    let home = Home::new("state-parked");
    let dir = home.state_dir();
    std::fs::create_dir_all(&dir).unwrap();
    for (pid, pct, seconds_ago) in [(9, 30, 20), (10, 50, 10)] {
        let file = dir.join(format!("state.{}.changes", pid));
        std::fs::write(&file, format!("q5_pct@x {}\n", pct)).unwrap();
        let at = SystemTime::now() - Duration::from_secs(seconds_ago);
        let parked = std::fs::File::options().write(true).open(&file).unwrap();
        parked.set_modified(at).unwrap();
    }
    let (five, seven) = limits(0, now());
    render(home.session(0), &input(five, seven));
    let state = std::fs::read_to_string(dir.join("state")).unwrap();
    assert_eq!(lines_of(&state, "x"), ["q5_pct@x 50"], "{}", state);
}
