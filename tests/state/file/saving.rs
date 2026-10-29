use super::{input, limits, now, render, ACCOUNTS};
use crate::common::Home;

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
