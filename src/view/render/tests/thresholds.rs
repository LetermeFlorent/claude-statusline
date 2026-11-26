use super::common::{line, sample};
use crate::config::Metric;
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
        colour(&ctx_data(89.6), "ctx"),
        colour(&ctx_data(90.0), "ctx")
    );
    assert_ne!(
        colour(&ctx_data(89.4), "ctx"),
        colour(&ctx_data(90.0), "ctx")
    );
}

#[test]
fn system_colour_follows_the_rounded_value() {
    assert_eq!(
        colour(&cpu_data(90.4), "cpu"),
        colour(&cpu_data(90.0), "cpu")
    );
    assert_eq!(
        colour(&cpu_data(50.4), "cpu"),
        colour(&cpu_data(50.0), "cpu")
    );
    assert_ne!(
        colour(&cpu_data(50.6), "cpu"),
        colour(&cpu_data(50.0), "cpu")
    );
}
