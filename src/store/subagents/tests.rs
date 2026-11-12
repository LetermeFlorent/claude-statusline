use super::*;

fn opt(s: &str) -> Option<String> {
    Some(String::from(s))
}

#[test]
fn a_full_task_is_read() {
    let text = r#"{"columns":80,"hook_event_name":"x","tasks":[{"id":"a1","name":"scout","type":"local_agent","agentType":"Explore","status":"running","description":"d","label":"Looking","startTime":1,"model":"claude-opus-4-7","effort":"high","tokenCount":5,"tokenSamples":[1],"cwd":"/"}]}"#;
    let p = parse(text).unwrap();
    assert_eq!(p.columns, Some(80));
    assert_eq!(
        p.tasks,
        vec![Task {
            id: String::from("a1"),
            name: opt("scout"),
            label: opt("Looking"),
            status: opt("running"),
            model: opt("claude-opus-4-7"),
            effort: opt("high"),
        }]
    );
}

#[test]
fn a_missing_model_stays_none() {
    let p = parse(r#"{"columns":60,"tasks":[{"id":"a","effort":"low"}]}"#).unwrap();
    assert_eq!(p.tasks[0].model, None);
    assert_eq!(p.tasks[0].effort, opt("low"));
}

#[test]
fn a_missing_effort_stays_none() {
    let p = parse(r#"{"columns":60,"tasks":[{"id":"a","model":"m"}]}"#).unwrap();
    assert_eq!(p.tasks[0].effort, None);
    assert_eq!(p.tasks[0].model, opt("m"));
}

#[test]
fn a_numeric_effort_becomes_its_decimal_text() {
    let p = parse(r#"{"columns":60,"tasks":[{"id":"a","effort":8192}]}"#).unwrap();
    assert_eq!(p.tasks[0].effort, opt("8192"));
}

#[test]
fn empty_tasks_give_an_empty_list() {
    assert_eq!(parse(r#"{"columns":60,"tasks":[]}"#).unwrap().tasks, vec![]);
    assert_eq!(parse(r#"{"columns":60}"#).unwrap().tasks, vec![]);
}

#[test]
fn invalid_json_is_an_error() {
    assert_eq!(parse("not json"), Err(Invalid));
    assert_eq!(parse(""), Err(Invalid));
}
