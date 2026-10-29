use super::*;
use crate::messages;
use crate::plain;

const NOW: f64 = 1_000_000.0;
const WEEK_RESET_SECS: f64 = 200_000.0;

fn data() -> Data {
    Data {
        ctx: Ctx::default(),
        window: None,
        quotas: Some(Quotas {
            five_hour: Some(Quota {
                pct: Some(36.0),
                at: Some(NOW + WEEK_RESET_SECS),
            }),
            seven_day: Some(Quota {
                pct: Some(81.0),
                at: Some(NOW + WEEK_RESET_SECS),
            }),
        }),
        model: String::from("Opus 4"),
        effort: None,
        sys: alloc::vec![SysValue {
            metric: Metric::Ram,
            pct: None,
            ram: Some((15.3 * GIB, 4.0 * GIB)),
        }],
        updating: true,
    }
}

fn bar() -> String {
    let cfg = Config::parse(None);
    let rendered = render(&cfg, &cfg.colors(false), &data(), NOW, &Terminal::WIDE);
    plain::strip(&rendered)
}

#[test]
fn english_keeps_its_units() {
    let text = bar();
    assert!(text.contains("7d"), "{}", text);
    assert!(text.contains("2d07h"), "{}", text);
    assert!(text.contains("55h33m"), "{}", text);
    assert!(text.contains("11.3/15.3G"), "{}", text);
    assert!(text.contains("update"), "{}", text);
}

#[test]
fn french_bar_units_are_french() {
    let text = messages::in_french(bar);
    assert!(text.contains("7j"), "{}", text);
    assert!(text.contains("2j07h"), "{}", text);
    assert!(text.contains("55h33min"), "{}", text);
    assert!(text.contains("11,3/15,3Go"), "{}", text);
    assert!(text.contains("maj"), "{}", text);
    assert!(!text.contains("7d") && !text.contains("update"), "{}", text);
}

#[test]
fn the_day_format_follows_the_window_not_its_label() {
    assert_eq!(reset_text(WEEK_RESET_SECS, false), "55h33m");
    assert_eq!(reset_text(WEEK_RESET_SECS, true), "2d07h");
    messages::in_french(|| {
        assert_eq!(reset_text(WEEK_RESET_SECS, false), "55h33min");
        assert_eq!(reset_text(WEEK_RESET_SECS, true), "2j07h");
    });
}

#[test]
fn french_minutes_and_billions_use_their_own_words() {
    messages::in_french(|| {
        assert_eq!(reset_text(59.9, false), "0min");
        assert_eq!(reset_text(600.0, true), "10min");
        assert_eq!(tokens(999_500_000), "1Md");
        assert_eq!(tokens(1_499_000), "1M");
        assert_eq!(tokens(60_000), "60k");
    });
    assert_eq!(tokens(999_500_000), "1B");
    assert_eq!(reset_text(600.0, true), "10m");
}
