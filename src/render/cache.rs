use super::*;
use crate::messages;
use crate::plain;

const NOW: f64 = 1_000_000.0;
const MARKER_ON: &str = "{\"cold_cache_marker\": true}";

fn data(cache_warm: Option<bool>) -> Data {
    Data {
        ctx: Ctx {
            size: Some(200_000.0),
            pct: Some(30.0),
            tokens: Some(60_000),
        },
        window: None,
        quotas: None,
        spend: None,
        cache_warm,
        model: String::from("Opus 4"),
        effort: None,
        sys: Vec::new(),
        updating: false,
    }
}

fn bar(config: Option<&str>, cache_warm: Option<bool>) -> String {
    let cfg = Config::parse(config);
    let data = data(cache_warm);
    let rendered = render(&cfg, &cfg.colors(false), &data, NOW, &Terminal::WIDE);
    plain::strip(&rendered)
}

#[test]
fn a_cold_cache_is_marked_when_the_option_is_on() {
    let text = bar(Some(MARKER_ON), Some(false));
    assert!(text.contains("60k/200k cold"), "{}", text);
}

#[test]
fn a_warm_or_unknown_cache_is_never_marked() {
    for warm in [Some(true), None] {
        let text = bar(Some(MARKER_ON), warm);
        assert!(!text.contains("cold"), "{}", text);
    }
}

#[test]
fn the_marker_is_off_by_default() {
    let text = bar(None, Some(false));
    assert!(!text.contains("cold"), "{}", text);
    let off = bar(Some("{\"cold_cache_marker\": false}"), Some(false));
    assert!(!off.contains("cold"), "{}", off);
}

#[test]
fn the_marker_word_is_translated() {
    let text = messages::in_french(|| bar(Some(MARKER_ON), Some(false)));
    assert!(text.contains("60k/200k froid"), "{}", text);
}
