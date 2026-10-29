use crate::messages;
use crate::plain;
use crate::render::*;

const NOW: f64 = 1_000_000.0;
const THREE_DAYS: f64 = 259_200.0;

fn spend_only() -> Data {
    Data {
        ctx: Ctx::default(),
        window: None,
        quotas: None,
        cache_warm: None,
        spend: Some(Quota {
            pct: Some(63.0),
            at: Some(NOW + THREE_DAYS + 7200.0),
        }),
        model: String::from("Opus 4"),
        effort: None,
        sys: Vec::new(),
        updating: false,
    }
}

fn bar(config: Option<&str>, data: &Data) -> String {
    let cfg = Config::parse(config);
    let rendered = render(&cfg, &cfg.colors(false), data, NOW, &Terminal::WIDE);
    plain::strip(&rendered)
}

#[test]
fn a_session_with_only_a_spend_limit_shows_the_spend_segment() {
    let text = bar(None, &spend_only());
    assert!(text.contains("spend"), "{}", text);
    assert!(text.contains("63%"), "{}", text);
    assert!(text.contains("3d02h"), "{}", text);
    assert!(!text.contains("5h") && !text.contains("7d"), "{}", text);
}

#[test]
fn the_spend_segment_is_drawn_like_a_quota() {
    let mut data = spend_only();
    let text = bar(None, &data);
    data.spend = None;
    data.quotas = Some(Quotas {
        five_hour: Some(Quota {
            pct: Some(63.0),
            at: None,
        }),
        seven_day: None,
    });
    let only_5h = "{\"segments\": {\"7d\": false}}";
    let quota = bar(Some(only_5h), &data);
    let drawn = |text: &str, label: &str| {
        let from = text.find(label).unwrap() + label.len();
        text[from..].split("63%").next().unwrap().to_string()
    };
    assert_eq!(
        drawn(&text, "spend"),
        drawn(&quota, "5h"),
        "{} {}",
        text,
        quota
    );
}

#[test]
fn real_quotas_hide_the_spend_segment() {
    let mut data = spend_only();
    data.quotas = Some(Quotas {
        five_hour: Some(Quota {
            pct: Some(10.0),
            at: None,
        }),
        seven_day: None,
    });
    let text = bar(None, &data);
    assert!(text.contains("5h") && !text.contains("spend"), "{}", text);
}

#[test]
fn the_segments_setting_hides_the_spend_segment() {
    let config = "{\"segments\": {\"spend\": false}}";
    let text = bar(Some(config), &spend_only());
    assert!(!text.contains("spend"), "{}", text);
}

#[test]
fn the_spend_label_is_translated() {
    let text = messages::in_french(|| bar(None, &spend_only()));
    assert!(text.contains("dépense"), "{}", text);
    assert!(text.contains("3j02h"), "{}", text);
}
