use super::*;
use crate::plain;
use crate::render::screen::fit::widest;

fn task(name: Option<&str>, model: Option<&str>, effort: Option<&str>) -> Task {
    Task {
        id: String::from("id1"),
        name: name.map(String::from),
        label: Some(String::from("Looking around")),
        status: None,
        model: model.map(String::from),
        effort: effort.map(String::from),
    }
}

fn raw_row(task: &Task, columns: usize) -> String {
    let cfg = Config::parse(None);
    let paint = Paint {
        cfg: &cfg,
        col: &cfg.colors(false),
        depth: Depth::TrueColor,
    };
    row(&paint, task, columns)
}

fn plain_row(task: &Task, columns: usize) -> String {
    let text = raw_row(task, columns);
    assert!(widest(&text) <= columns);
    plain::strip(&text)
}

#[test]
fn model_and_effort_are_joined_by_a_colon() {
    let t = task(Some("scout"), Some("claude-opus-4-8"), Some("high"));
    assert_eq!(plain_row(&t, 80), "scout Opus 4.8:high");
}

#[test]
fn effort_gets_its_level_colour() {
    let t = task(None, Some("claude-opus-4-8"), Some("high"));
    let raw = raw_row(&t, 80);
    let low = raw_row(&task(None, Some("claude-opus-4-8"), Some("low")), 80);
    assert!(raw.contains("\x1b[38;2;"));
    assert_ne!(raw, low);
}

#[test]
fn model_alone_hides_the_colon() {
    let t = task(None, Some("claude-sonnet-4-5-20250929"), None);
    assert_eq!(plain_row(&t, 80), "Sonnet 4.5");
}

#[test]
fn effort_alone_hides_the_model() {
    let t = task(None, None, Some("max"));
    assert_eq!(plain_row(&t, 80), "max");
    let named = task(Some("scout"), None, Some("max"));
    assert_eq!(plain_row(&named, 80), "scout max");
}

#[test]
fn without_model_or_effort_the_name_then_label_then_id_show() {
    assert_eq!(plain_row(&task(Some("scout"), None, None), 80), "scout");
    assert_eq!(plain_row(&task(None, None, None), 80), "Looking around");
    let mut t = task(None, None, None);
    t.label = None;
    assert_eq!(plain_row(&t, 80), "id1");
}

#[test]
fn a_numeric_effort_is_shown_as_text() {
    let t = task(None, Some("claude-haiku-4-5"), Some("12000"));
    assert_eq!(plain_row(&t, 80), "Haiku 4.5:12000");
}

#[test]
fn an_unknown_model_id_is_kept() {
    let t = task(None, Some("gpt-x"), Some("low"));
    assert_eq!(plain_row(&t, 80), "gpt-x:low");
}

#[test]
fn a_narrow_width_cuts_the_row() {
    let t = task(Some("scout"), Some("claude-opus-4-8"), Some("high"));
    assert_eq!(plain_row(&t, 12), "scout Opus 4");
    assert_eq!(plain_row(&t, 6), "scout");
    assert_eq!(plain_row(&t, 3), "sco");
    assert_eq!(plain_row(&t, 0), "");
    let t = task(None, Some("claude-opus-4-8"), Some("high"));
    assert_eq!(plain_row(&t, 8), "Opus 4.8");
    assert_eq!(plain_row(&t, 10), "Opus 4.8:h");
}
