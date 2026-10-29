#![cfg(windows)]

mod common;

use common::Home;
use std::io::Write;
use std::process::{Command, Output};
use std::time::{Duration, Instant};

fn input(folder: &str) -> String {
    let folder = folder.replace('\\', "\\\\");
    format!(
        "{{\"model\":{{\"display_name\":\"Opus\"}},\"cwd\":\"{0}\",\"workspace\":{{\"project_dir\":\"{0}\"}},\"context_window\":{{\"context_window_size\":200000,\"used_percentage\":30}}}}",
        folder
    )
}

fn render(command: &mut Command, input: &str) -> (Output, Duration) {
    let start = Instant::now();
    let mut child = command.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    (output, start.elapsed())
}

fn assert_rendered(output: &Output) {
    let errors = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{}", errors);
    assert!(errors.is_empty(), "{}", errors);
    let shown = String::from_utf8_lossy(&output.stdout);
    assert!(shown.contains("Opus"), "{}", shown);
}

#[test]
fn a_session_on_an_unreachable_share_renders_at_once() {
    const UNREACHABLE_SHARE: &str = r"\\192.0.2.1\share";
    const BOUND: Duration = Duration::from_millis(300);
    let home = Home::new("share");
    let local = home.0.to_string_lossy().into_owned();
    assert_rendered(&render(&mut home.session(0), &input(&local)).0);
    let (output, spent) = render(
        home.session(0).env("TERM_PROGRAM", "vscode"),
        &input(UNREACHABLE_SHARE),
    );
    assert_rendered(&output);
    assert!(spent < BOUND, "{:?}", spent);
}
