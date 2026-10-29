mod position;
mod schema;
#[cfg(test)]
mod tests;
mod walk;

pub use walk::problems;

use crate::config::{BACKGROUND_NAMES, EFFORT_NAMES, METRIC_NAMES};
use crate::messages;
use crate::os::{self, Contents};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use schema::Kind;
use walk::Fault;

const FAILED_EXIT: i32 = 1;

pub fn run(path: Option<String>) -> i32 {
    let Some(path) = path else {
        return os::fail(messages::NO_HOME.text(), FAILED_EXIT);
    };
    let text = match os::read_checked(&path, os::MAX_READ_BYTES) {
        Contents::Bytes(raw) => String::from_utf8_lossy(&raw).into_owned(),
        Contents::Missing => {
            os::out(&messages::CHECK_MISSING.with(&[&path]));
            return 0;
        }
        Contents::Unreadable => {
            let message = messages::CHECK_UNREADABLE.with(&[&path]);
            return os::fail(&message, FAILED_EXIT);
        }
    };
    let found = problems(&text);
    if found.is_empty() {
        os::out(&messages::CHECK_CLEAN.with(&[&path]));
        return 0;
    }
    let mut report = String::new();
    for problem in &found {
        let (line, column) = position::line_and_column(&text, problem.at);
        let fault = describe(&problem.fault);
        report.push_str(&format!("{}:{}:{}: {}", path, line, column, fault));
    }
    os::out(&report);
    1
}

fn describe(fault: &Fault) -> String {
    match fault {
        Fault::InvalidJson => String::from(messages::INVALID_JSON.text()),
        Fault::NotAnObject => String::from(messages::NOT_AN_OBJECT.text()),
        Fault::TrailingText => String::from(messages::TRAILING_TEXT.text()),
        Fault::UnknownKey(path) => messages::UNKNOWN_KEY.with(&[path]),
        Fault::Expects(path, kind) => messages::EXPECTS.with(&[path, &expected(kind)]),
    }
}

fn expected(kind: &Kind) -> String {
    let text = match kind {
        Kind::Flag => messages::EXPECTED_FLAG,
        Kind::Number => messages::EXPECTED_NUMBER,
        Kind::Text => messages::EXPECTED_TEXT,
        Kind::Color => messages::EXPECTED_COLOR,
        Kind::List(_) => messages::EXPECTED_LIST,
        Kind::Object(_) => messages::EXPECTED_OBJECT,
        Kind::Background => return one_of(&BACKGROUND_NAMES),
        Kind::Effort => return one_of(&EFFORT_NAMES),
        Kind::Metric => return one_of(&METRIC_NAMES),
    };
    String::from(text.text())
}

fn one_of<T>(names: &[(&str, T)]) -> String {
    let names: Vec<&str> = names.iter().map(|(name, _)| *name).collect();
    messages::EXPECTED_ONE_OF.with(&[&names.join(", ")])
}
