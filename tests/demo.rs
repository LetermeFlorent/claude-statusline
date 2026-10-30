mod common;

use common::Home;
use std::process::Output;

const DISK_LABELS: [&str; 2] = ["dsk", "C:"];

fn demo(arguments: &[&str]) -> Output {
    let home = Home::new("demo");
    home.session(0).args(arguments).output().unwrap()
}

fn shown(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn percent_after(text: &str, label: &str) -> Option<String> {
    let mut words = text.split_whitespace().skip_while(|word| *word != label);
    words.nth(2).map(String::from)
}

fn disk_percent(text: &str) -> Option<String> {
    DISK_LABELS
        .iter()
        .find_map(|label| percent_after(text, label))
}

#[test]
fn the_demo_shows_a_disk_and_a_cpu_percentage() {
    let text = shown(&demo(&["--demo"]));
    assert_eq!(disk_percent(&text).as_deref(), Some("12%"), "{}", text);
    assert_eq!(
        percent_after(&text, "cpu").as_deref(),
        Some("18%"),
        "{}",
        text
    );
}

#[test]
fn the_demo_levels_cross_the_warning_and_the_hot_values() {
    let warn = shown(&demo(&["--demo", "warn"]));
    assert_eq!(disk_percent(&warn).as_deref(), Some("76%"), "{}", warn);
    assert_eq!(
        percent_after(&warn, "cpu").as_deref(),
        Some("78%"),
        "{}",
        warn
    );
    let hot = shown(&demo(&["--demo", "HOT"]));
    assert_eq!(disk_percent(&hot).as_deref(), Some("93%"), "{}", hot);
    assert_eq!(
        percent_after(&hot, "cpu").as_deref(),
        Some("96%"),
        "{}",
        hot
    );
}

#[test]
fn an_unknown_demo_level_is_refused() {
    let output = demo(&["--demo", "medium"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
}
