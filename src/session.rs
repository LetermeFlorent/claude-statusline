use crate::json::{self, Value};
use crate::num;
use crate::render::{Ctx, Quota};
use alloc::string::String;

// Ce que Claude Code envoie sur l'entree standard
pub struct Session {
    pub model: String,
    pub effort: Option<String>,
    pub ctx: Ctx,
    // Some des que five_hour ou seven_day est present, meme nul : les quotas viennent alors de Claude Code
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
        Quota { pct: self.pct, at: self.at }
    }
}

fn model_name(m: Option<&Value>) -> String {
    let m = m.filter(|v| v.is_obj());
    match m.and_then(|m| m.get("display_name")).and_then(|v| v.str()) {
        Some(s) if !s.is_empty() && s != "Unknown" => {
            let cut = s.find('(').map(|i| &s[..i]).unwrap_or(s);
            String::from(cut.trim())
        }
        _ => String::from(m.and_then(|m| m.get("id")).and_then(|v| v.str()).unwrap_or("claude")),
    }
}

// Somme des jetons d'entree ; un champ present mais invalide annule tout
fn usage_tokens(cu: Option<&Value>) -> Option<u64> {
    let cu = cu?;
    if *cu == Value::Null {
        return None;
    }
    let mut sum = 0u64;
    for k in ["input_tokens", "cache_creation_input_tokens", "cache_read_input_tokens"] {
        if let Some(v) = cu.get(k) {
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
        let v = json::parse(text, false).unwrap_or(Value::Null);
        let cw = v.get("context_window");
        let rl = v.get("rate_limits");
        let five = rl.and_then(|r| r.get("five_hour"));
        let seven = rl.and_then(|r| r.get("seven_day"));
        Session {
            model: model_name(v.get("model")),
            effort: v
                .at(&["effort", "level"])
                .and_then(|x| x.str())
                .map(|s| String::from(s.trim()))
                .filter(|s| !s.is_empty()),
            ctx: Ctx {
                size: cw.and_then(|c| c.get("context_window_size")).and_then(|x| x.num()),
                pct: cw.and_then(|c| c.get("used_percentage")).and_then(|x| x.num()),
                tokens: usage_tokens(cw.and_then(|c| c.get("current_usage"))),
            },
            live: if five.is_some() || seven.is_some() { Some((live(five), live(seven))) } else { None },
            cwd: v.get("cwd").and_then(|x| x.str()).map(String::from),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lecture() {
        let s = Session::parse("{\"model\":{\"display_name\":\"Opus 4 (1M context)\"},\"effort\":{\"level\":\" high \"}}");
        assert_eq!(s.model, "Opus 4");
        assert_eq!(s.effort.as_deref(), Some("high"));
        assert_eq!(Session::parse("{\"model\":{\"display_name\":\"(x)\",\"id\":\"i\"}}").model, "");
        assert_eq!(Session::parse("{\"model\":{\"display_name\":\"Unknown\",\"id\":\"i\"}}").model, "i");
        assert_eq!(Session::parse("{\"model\":\"Opus\"}").model, "claude");
        let t = |cu: &str| Session::parse(&alloc::format!("{{\"context_window\":{{\"current_usage\":{}}}}}", cu)).ctx.tokens;
        assert_eq!(t("{\"input_tokens\":1000.0,\"cache_read_input_tokens\":1.5e3}"), Some(2500));
        assert_eq!(t("{\"input_tokens\":1000.7}"), None);
        assert_eq!(t("{\"input_tokens\":null}"), None);
        assert_eq!(t("[1]"), Some(0));
        assert_eq!(t("null"), None);
        assert!(Session::parse("{\"rate_limits\":{}}").live.is_none());
        assert!(Session::parse("{\"rate_limits\":{\"five_hour\":null}}").live.is_some());
    }
}
