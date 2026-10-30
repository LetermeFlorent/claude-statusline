use super::named;
use crate::json::Value;
use crate::theme::{Lookup, Mode};

pub fn zed(settings: &Value, lookup: &Lookup<'_>) -> Mode {
    match settings.get("theme") {
        Some(Value::Str(name)) => named(name, lookup),
        Some(choice) if choice.is_obj() => match choice.get("mode").and_then(|m| m.str()) {
            Some("light") => Mode::Light,
            Some("dark") => Mode::Dark,
            _ => Mode::System,
        },
        _ => Mode::System,
    }
}
