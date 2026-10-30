use super::*;

#[test]
fn values_are_clamped_and_bad_ones_ignored() {
    let c = Config::parse(Some("{\"bar_width\": 2.7, \"bold\": 0, \"glyphs\": {\"empty\": \"\"}, // x\n \"system\": {\"metrics\": [\"io\", \"RAM\", \"io\"]},}"));
    assert_eq!(c.width, 2);
    assert!(c.bold);
    assert_eq!(c.empty, "");
    assert_eq!(c.metrics, [Metric::Io, Metric::Ram]);
    assert_eq!(Config::parse(Some("{\"bar_width\": 1000}")).width, 40);
    assert_eq!(Config::parse(Some("{\"bar_width\": -3}")).width, 1);
    assert_eq!(Config::parse(Some("[1]")).width, 8);
    assert_eq!(
        hex(Some(&Value::Str(String::from(" #abc ")))),
        Some([170.0, 187.0, 204.0])
    );
    assert_eq!(hex(Some(&Value::Str(String::from("#GG0000")))), None);
}

fn metrics_of(list: &str) -> Vec<Metric> {
    let text = alloc::format!("{{\"system\": {{\"metrics\": {}}}}}", list);
    Config::parse(Some(&text)).metrics
}

#[test]
fn metric_names_ignore_case_and_take_dsk_for_the_disk() {
    assert_eq!(metrics_of("[\"Ram\"]"), [Metric::Ram]);
    assert_eq!(
        metrics_of("[\" CPU \", \"dsk\"]"),
        [Metric::Cpu, Metric::Io]
    );
}

#[test]
fn a_metric_listed_twice_shows_once() {
    assert_eq!(metrics_of("[\"ram\", \"ram\"]"), [Metric::Ram]);
    assert_eq!(
        metrics_of("[\"io\", \"cpu\", \"dsk\"]"),
        [Metric::Io, Metric::Cpu]
    );
}

#[test]
fn a_list_with_no_known_name_shows_every_metric() {
    let every = [Metric::Ram, Metric::Io, Metric::Cpu];
    assert_eq!(metrics_of("[\"gpu\", \"net\"]"), every);
    assert_eq!(metrics_of("[]"), every);
    assert_eq!(metrics_of("\"ram\""), every);
}

#[test]
fn effort_style_ignores_case() {
    let flat = |text: &str| Config::parse(Some(text)).effort_flat;
    assert!(flat("{\"effort_style\": \"Flat\"}"));
    assert!(!flat("{\"effort_style\": \"gradient\"}"));
}

#[test]
fn automatic_updates_stay_off_unless_turned_on() {
    let auto_update = |text: &str| Config::parse(Some(text)).auto_update;
    assert!(!Config::parse(None).auto_update);
    assert!(!auto_update("{}"));
    assert!(!auto_update("{\"auto_update\": false}"));
    assert!(!auto_update("{\"auto_update\": \"yes\"}"));
    assert!(auto_update("{\"auto_update\": true}"));
}

const EVERY_KEY: &str = r##"{
    "gradient": {"5h": 0.1, "7d": 2, "x": 1},
    "thresholds": {"ctx_warn": 1, "ctx_hot": 2, "quota_warn": 3,
        "quota_hot": 4, "system_warn": 5, "system_hot": 6},
    "palettes": {"5h": {"ok": "#010203", "warn": "#040506", "hot": "#070809"},
        "7d": {"ok": "#0a0b0c", "warn": "#0d0e0f", "hot": "#101112"}},
    "segments": {"ctx": false, "5h": false, "7d": false, "model": false},
    "system": {"gradient": -1, "disk_path": "D:\\", "enabled": false,
        "colors": {"ram": {"ok": "#111111"}, "dsk": {"warn": "#222222"},
            "cpu": {"hot": "#333333"}}}
}"##;

#[test]
fn every_key_reaches_its_field() {
    let c = Config::parse(Some(EVERY_KEY));
    assert_eq!([c.grad_5h, c.grad_7d, c.grad_sys], [0.1, 1.0, 0.0]);
    assert_eq!([c.ctx_warn, c.ctx_hot], [1.0, 2.0]);
    assert_eq!([c.quota_warn, c.quota_hot], [3.0, 4.0]);
    assert_eq!([c.sys_warn, c.sys_hot], [5.0, 6.0]);
    assert_eq!(c.pal_5h.ok, [1.0, 2.0, 3.0]);
    assert_eq!(c.pal_5h.hot, [7.0, 8.0, 9.0]);
    assert_eq!(c.pal_7d.warn, [13.0, 14.0, 15.0]);
    assert_eq!(c.pal_7d.hot, [16.0, 17.0, 18.0]);
    assert_eq!(c.pal_ram.ok, [17.0, 17.0, 17.0]);
    assert_eq!(c.pal_dsk.warn, [34.0, 34.0, 34.0]);
    assert_eq!(c.pal_cpu.hot, [51.0, 51.0, 51.0]);
    assert_eq!(c.pal_ram.warn, WARN_SYS);
    assert_eq!(c.disk_path, "D:\\");
    assert!(!c.sys_enabled);
    assert!(!c.seg_ctx && !c.seg_5h && !c.seg_7d && !c.seg_model);
}

#[test]
fn missing_keys_keep_their_defaults() {
    let c = Config::parse(None);
    assert_eq!([c.grad_5h, c.grad_7d, c.grad_sys], [0.55, 0.3, 0.45]);
    assert_eq!([c.ctx_warn, c.ctx_hot], [70.0, 90.0]);
    assert_eq!([c.quota_warn, c.quota_hot], [30.0, 70.0]);
    assert_eq!([c.sys_warn, c.sys_hot], [70.0, 85.0]);
    assert_eq!(c.pal_5h.warn, [170.0, 90.0, 175.0]);
    assert_eq!(c.pal_7d.ok, [83.0, 137.0, 119.0]);
    assert_eq!(c.pal_ram.ok, [30.0, 120.0, 125.0]);
    assert_eq!(c.pal_dsk.ok, [105.0, 120.0, 85.0]);
    assert_eq!(c.pal_cpu.hot, HOT_SYS);
    assert_eq!(c.disk_path, DEFAULT_DISK);
    assert!(c.sys_enabled && c.seg_ctx && c.seg_model);
}

#[test]
fn glyphs_lose_terminal_controls() {
    let config = Config::parse(Some(
        r#"{"glyphs": {"filled": "\u001b[5m#", "half": "\u202e+", "empty": "\u0085."}, "auto_compact_marker": "\u0007*"}"#,
    ));
    assert_eq!(
        [config.filled, config.half, config.empty, config.marker],
        ["[5m#", "+", ".", "*"]
    );
}
