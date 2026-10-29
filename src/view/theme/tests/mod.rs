use crate::json::{self, Syntax, Value};
use crate::theme::Mode;

mod fixtures;
mod hosts;
mod settings;

fn j(s: &str) -> Value {
    json::parse(s, Syntax::Commented).unwrap()
}

fn none(_: &str) -> Option<Mode> {
    None
}
