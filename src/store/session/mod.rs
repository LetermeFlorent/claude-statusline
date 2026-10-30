use crate::json::{self, Syntax, Value};
use crate::num;
use crate::render::{label_text, Ctx, Quota};
use alloc::string::String;

pub struct Session {
    pub model: String,
    pub model_id: Option<String>,
    pub effort: Option<String>,
    pub ctx: Ctx,
    pub live: Option<(Option<Live>, Option<Live>)>,
    pub spend: Option<Live>,
    pub cache_warm: Option<bool>,
    pub cwd: Option<String>,
    pub project: Option<String>,
}

#[derive(Clone, Copy)]
pub struct Live {
    pub pct: Option<f64>,
    pub at: Option<f64>,
}

impl Live {
    pub fn quota(&self) -> Quota {
        Quota {
            pct: self.pct,
            at: self.at,
        }
    }
}

fn model_name(model: Option<&Value>) -> String {
    let model = model.filter(|v| v.is_obj());
    match model
        .and_then(|m| m.get("display_name"))
        .and_then(|v| v.str())
    {
        Some(s) if !s.is_empty() && s != "Unknown" => {
            let cut = s.find('(').map(|i| &s[..i]).unwrap_or(s);
            label_text(cut)
        }
        _ => label_text(
            model
                .and_then(|m| m.get("id"))
                .and_then(|v| v.str())
                .unwrap_or("claude"),
        ),
    }
}

fn usage_tokens(current_usage: Option<&Value>) -> Option<u64> {
    let current_usage = current_usage?;
    if *current_usage == Value::Null {
        return None;
    }
    let mut sum = 0u64;
    for k in [
        "input_tokens",
        "cache_creation_input_tokens",
        "cache_read_input_tokens",
    ] {
        if let Some(v) = current_usage.get(k) {
            let n = v.num()?;
            if n < 0.0 || !num::is_integral(n) {
                return None;
            }
            sum = sum.saturating_add(n as u64);
        }
    }
    Some(sum)
}

fn context_tokens(context_window: Option<&Value>) -> Option<u64> {
    let context_window = context_window?;
    match context_window.get("current_usage") {
        None | Some(Value::Null) => context_window
            .get("total_input_tokens")
            .and_then(|v| v.num())
            .filter(|n| *n >= 0.0 && num::is_integral(*n))
            .map(|n| n as u64),
        usage => usage_tokens(usage),
    }
}

fn live(v: Option<&Value>) -> Option<Live> {
    let v = v.filter(|v| v.is_obj())?;
    Some(Live {
        pct: v.get("used_percentage").and_then(|x| x.num()),
        at: v.get("resets_at").and_then(|x| x.num()),
    })
}

impl Session {
    pub fn parse(text: &str) -> Session {
        let payload = json::parse(text, Syntax::Tolerant).unwrap_or(Value::Null);
        let context_window = payload.get("context_window");
        let limits = payload.get("rate_limits");
        let five = limits.and_then(|r| r.get("five_hour"));
        let seven = limits.and_then(|r| r.get("seven_day"));
        Session {
            model: model_name(payload.get("model")),
            model_id: payload
                .at(&["model", "id"])
                .and_then(|x| x.str())
                .map(String::from),
            effort: payload
                .at(&["effort", "level"])
                .and_then(|x| x.str())
                .map(label_text)
                .filter(|s| !s.is_empty()),
            ctx: Ctx {
                size: context_window
                    .and_then(|c| c.get("context_window_size"))
                    .and_then(|x| x.num()),
                pct: context_window
                    .and_then(|c| c.get("used_percentage"))
                    .and_then(|x| x.num()),
                tokens: context_tokens(context_window),
            },
            live: if five.is_some() || seven.is_some() {
                Some((live(five), live(seven)))
            } else {
                None
            },
            cache_warm: payload.at(&["prompt_cache", "warm"]).and_then(|x| x.bool()),
            spend: live(limits.and_then(|r| r.get("spend_limit"))),
            cwd: payload.get("cwd").and_then(|x| x.str()).map(String::from),
            project: payload
                .at(&["workspace", "project_dir"])
                .and_then(|x| x.str())
                .map(String::from),
        }
    }
}

#[cfg(test)]
mod tests;
