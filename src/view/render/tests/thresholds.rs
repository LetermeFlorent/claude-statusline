use super::common::{line, sample};
use crate::config::Metric;
use crate::render::text::pct_text;
use crate::render::{Ctx, Data, Quota, Quotas, SysValue};

const CONFIG: &str = r#"{"thresholds": {"quota_warn": 50, "quota_hot": 90, "ctx_warn": 50, "ctx_hot": 90, "system_warn": 50, "system_hot": 90}}"#;

fn first_colour(rendered: &str, anchor: &str) -> String {
    let tail = rendered.split(anchor).nth(1).unwrap();
    let start = tail.find("[#").unwrap();
    String::from(&tail[start..start + 9])
}

fn quota_data(pct: f64) -> Data {
    let mut data = sample();
    data.quotas = Some(Quotas {
        five_hour: Some(Quota {
            pct: Some(pct),
            at: None,
        }),
        seven_day: None,
    });
    data
}

fn ctx_data(pct: f64) -> Data {
    let mut data = sample();
    data.window = None;
    data.ctx = Ctx {
        size: None,
        pct: Some(pct),
        tokens: None,
    };
    data
}

fn cpu_data(pct: f64) -> Data {
    let mut data = sample();
    data.sys = alloc::vec![SysValue {
        metric: Metric::Cpu,
        pct: Some(pct),
        ram: None,
    }];
    data
}

fn colour(data: &Data, anchor: &str) -> String {
    first_colour(&line(false, Some(CONFIG), data), anchor)
}

#[test]
fn quota_colour_follows_the_rounded_value() {
    assert_eq!(
        colour(&quota_data(89.96), "5h"),
        colour(&quota_data(90.0), "5h")
    );
    assert_ne!(
        colour(&quota_data(89.94), "5h"),
        colour(&quota_data(90.0), "5h")
    );
}

#[test]
fn context_colour_follows_the_rounded_value() {
    assert_eq!(
        colour(&ctx_data(89.96), "ctx"),
        colour(&ctx_data(90.0), "ctx")
    );
}

#[test]
fn system_colour_follows_the_rounded_value() {
    assert_eq!(
        colour(&cpu_data(90.04), "cpu"),
        colour(&cpu_data(90.0), "cpu")
    );
    assert_eq!(
        colour(&cpu_data(50.04), "cpu"),
        colour(&cpu_data(50.0), "cpu")
    );
}

#[test]
fn non_finite_percent_reads_dashes() {
    assert_eq!(pct_text(Some(f64::NAN)), "--%");
    assert_eq!(pct_text(Some(f64::INFINITY)), "--%");
    assert_eq!(pct_text(Some(f64::NEG_INFINITY)), "--%");
}

#[test]
fn huge_percent_is_clamped() {
    assert_eq!(pct_text(Some(1e300)), "999.9%");
    assert_eq!(pct_text(Some(f64::MAX)), "999.9%");
}

#[test]
fn negative_zero_percent_reads_zero() {
    assert_eq!(pct_text(Some(-0.0)), "0.0%");
    assert_eq!(pct_text(Some(-0.04)), "0.0%");
}
