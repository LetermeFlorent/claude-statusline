use crate::json::{self, Syntax, Value};
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, PartialEq)]
pub struct Invalid;

#[derive(Debug, PartialEq)]
pub struct Task {
    pub id: String,
    pub name: Option<String>,
    pub label: Option<String>,
    pub status: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct Payload {
    pub columns: Option<u64>,
    pub tasks: Vec<Task>,
}

fn text(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| x.str()).map(String::from)
}

fn effort(v: &Value) -> Option<String> {
    match v.get("effort")? {
        Value::Str(s) => Some(s.clone()),
        Value::Num(_, raw) => Some(raw.clone()),
        _ => None,
    }
}

fn task(v: &Value) -> Option<Task> {
    Some(Task {
        id: text(v, "id")?,
        name: text(v, "name"),
        label: text(v, "label"),
        status: text(v, "status"),
        model: text(v, "model"),
        effort: effort(v),
    })
}

pub fn parse(input: &str) -> Result<Payload, Invalid> {
    let payload = json::parse(input, Syntax::Tolerant)
        .filter(|v| v.is_obj())
        .ok_or(Invalid)?;
    Ok(Payload {
        columns: payload
            .get("columns")
            .and_then(|x| x.num())
            .filter(|n| *n >= 0.0)
            .map(|n| n as u64),
        tasks: payload
            .get("tasks")
            .and_then(|t| t.arr())
            .map(|t| t.iter().filter_map(task).collect())
            .unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests;
