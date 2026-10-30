use crate::json::{self, Syntax, Value};
use crate::num;
use crate::render::{label_text, Ctx, Quota};
use alloc::string::String;

pub struct Session {
    pub model: String,
    pub effort: Option<String>,
    pub ctx: Ctx,
    pub live: Option<(Option<Live>, Option<Live>)>,
    pub cwd: Option<String>,
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
                tokens: usage_tokens(context_window.and_then(|c| c.get("current_usage"))),
            },
            live: if five.is_some() || seven.is_some() {
                Some((live(five), live(seven)))
            } else {
                None
            },
            cwd: payload.get("cwd").and_then(|x| x.str()).map(String::from),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_model_effort_tokens_and_rate_limits() {
        let session = Session::parse("{\"model\":{\"display_name\":\"Opus 4 (1M context)\"},\"effort\":{\"level\":\" high \"}}");
        assert_eq!(session.model, "Opus 4");
        assert_eq!(session.effort.as_deref(), Some("high"));
        assert_eq!(
            Session::parse("{\"model\":{\"display_name\":\"(x)\",\"id\":\"i\"}}").model,
            ""
        );
        assert_eq!(
            Session::parse("{\"model\":{\"display_name\":\"Unknown\",\"id\":\"i\"}}").model,
            "i"
        );
        assert_eq!(Session::parse("{\"model\":\"Opus\"}").model, "claude");
        let tokens_for = |cu: &str| {
            Session::parse(&alloc::format!(
                "{{\"context_window\":{{\"current_usage\":{}}}}}",
                cu
            ))
            .ctx
            .tokens
        };
        assert_eq!(
            tokens_for("{\"input_tokens\":1000.0,\"cache_read_input_tokens\":1.5e3}"),
            Some(2500)
        );
        assert_eq!(tokens_for("{\"input_tokens\":1000.7}"), None);
        assert_eq!(tokens_for("{\"input_tokens\":null}"), None);
        assert_eq!(tokens_for("[1]"), Some(0));
        assert_eq!(tokens_for("null"), None);
        assert!(Session::parse("{\"rate_limits\":{}}").live.is_none());
        assert!(Session::parse("{\"rate_limits\":{\"five_hour\":null}}")
            .live
            .is_some());
    }

    #[test]
    fn terminal_controls_are_dropped_from_model_and_effort() {
        let session = Session::parse(
            r#"{"model":{"display_name":"Opus\u001b]52;c;aGk=\u0007"},"effort":{"level":"hi\u009bgh\u007f"}}"#,
        );
        assert_eq!(session.model, "Opus]52;c;aGk=");
        assert_eq!(session.effort.as_deref(), Some("high"));
    }

    #[test]
    fn bidi_controls_are_dropped() {
        let session = Session::parse(r#"{"model":{"id":"\u2066claude\u202e-opus\u2069"}}"#);
        assert_eq!(session.model, "claude-opus");
    }

    #[test]
    fn long_model_names_are_capped() {
        let name = "x".repeat(10_000);
        let session = Session::parse(&alloc::format!(
            r#"{{"model":{{"display_name":"{}"}}}}"#,
            name
        ));
        assert_eq!(
            session.model.chars().count(),
            crate::render::MAX_LABEL_CHARS
        );
    }
}
