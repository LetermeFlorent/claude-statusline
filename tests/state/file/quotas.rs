use super::{input, limits, lines_of, now, render, ACCOUNTS};
use crate::common::Home;
use std::io::Read;

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
