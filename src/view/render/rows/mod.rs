use super::line::Out;
use super::screen::fit;
use super::{label_text, Depth};
use crate::config::{Colors, Config};
use crate::render::text::tokens;
use crate::store::subagents::Task;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(test)]
mod tests;

const MAX_VERSION_DIGITS: usize = 2;

pub struct Paint<'a> {
    pub cfg: &'a Config,
    pub col: &'a Colors,
    pub depth: Depth,
    /// Alternates between two looks so that a rising count pulses.
    pub frame: bool,
}

fn short_model(id: &str) -> String {
    let Some(rest) = id.strip_prefix("claude-") else {
        return label_text(id);
    };
    let mut parts = rest.split('-');
    let family = parts.next().unwrap_or("");
    let known = matches!(family, "opus" | "sonnet" | "haiku");
    let version: Vec<&str> = parts
        .take_while(|p| !(p.len() > MAX_VERSION_DIGITS && p.chars().all(|c| c.is_ascii_digit())))
        .collect();
    let numeric = version.iter().all(|p| {
        !p.is_empty() && p.len() <= MAX_VERSION_DIGITS && p.chars().all(|c| c.is_ascii_digit())
    });
    if !known || !numeric {
        return label_text(id);
    }
    let mut name = String::new();
    name.extend(family.chars().take(1).flat_map(char::to_uppercase));
    name.push_str(&family[1..]);
    if !version.is_empty() {
        name.push(' ');
        name.push_str(&version.join("."));
    }
    name
}

fn cut(text: &str, budget: &mut usize) -> String {
    let kept: String = text.chars().take(*budget).collect();
    *budget -= kept.chars().count();
    kept
}

pub fn row(paint: &Paint, task: &Task, columns: usize) -> String {
    let given = |v: &Option<String>| v.clone().filter(|s| !s.is_empty());
    let mut name = given(&task.name)
        .map(|s| label_text(&s))
        .unwrap_or_default();
    let model = given(&task.model)
        .map(|s| short_model(&s))
        .unwrap_or_default();
    let effort = given(&task.effort)
        .map(|s| label_text(&s))
        .unwrap_or_default();
    if name.is_empty() && model.is_empty() && effort.is_empty() {
        name = given(&task.label)
            .map(|s| label_text(&s))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| label_text(&task.id));
    }
    let (sent, got) = task.usage.map_or_else(Default::default, |u| {
        (
            format!("\u{2191}{}", tokens(u.sent)),
            format!("\u{2193}{}", tokens(u.got)),
        )
    });
    // the counts sit at the right edge, so their width and one space are kept free
    let reserve = if sent.is_empty() {
        0
    } else {
        sent.chars().count() + got.chars().count() + 2
    };
    let (sent, got) = if reserve < columns {
        (sent, got)
    } else {
        Default::default()
    };
    let rising = task
        .usage
        .map_or((false, false), |u| (u.sent_rising, u.got_rising));
    let mut budget = columns - if sent.is_empty() { 0 } else { reserve };
    let name = cut(&name, &mut budget);
    let spacer = usize::from(!name.is_empty() && budget > 0);
    budget -= spacer.min(budget);
    let model = cut(&model, &mut budget);
    let colon = usize::from(!model.is_empty() && budget > 0);
    budget -= colon;
    let effort = if !model.is_empty() && colon == 0 {
        String::new()
    } else {
        cut(&effort, &mut budget)
    };
    let text = Out::row(
        paint,
        &name,
        &model,
        &effort,
        [(&sent, rising.0), (&got, rising.1)],
        columns,
    );
    debug_assert!(fit::widest(&text) <= columns);
    text
}
