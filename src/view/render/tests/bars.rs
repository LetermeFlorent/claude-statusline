use super::common::{line, sample};
use super::helpers::{ctx_only, glyphs, luminance, quota_only};
use crate::config::Config;
use alloc::string::String;

#[test]
fn quota_bar_levels() {
    let levels = [Some(0.0), Some(50.0), Some(80.0), Some(100.0), None].map(quota_only);
    assert_eq!(
        levels,
        [
            "[b]5h[/] [#55595f]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[/] [b]0%[/] [b]--[/][/]\n",
            "[b]5h[/] [#aa5aaf]■[#af63b3]■[#b46cb8]■[#b876bc]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[/] [b]50%[/] [b]--[/][/]\n",
            "[b]5h[/] [#c83c5f]■[#ca4163]■[#cb4768]■[#cd4c6c]■[#ce5271]■[#d05775]■[#d15c79]▪[#55595f]■[/] [b]80%[/] [b]--[/][/]\n",
            "[b]5h[/] [#c83c5f]■[#c94163]■[#cb4567]■[#cc4a6a]■[#cd4f6e]■[#cf5372]■[#d05876]■[#d15c79]■[/] [b][#d15c79]100%[/] [b]--[/][/]\n",
            "[b]5h[/] [#55595f]■■■■■■■■[/] [b]--%[/] [b]--[/][/]\n",
        ]
    );
}

#[test]
fn empty_cells_are_dimmer_than_full_ones_on_dark_and_lighter_on_light() {
    let cfg = Config::parse(None);
    let (dark, light) = (cfg.colors(false), cfg.colors(true));
    assert_eq!(dark.empty, [85.0, 89.0, 95.0]);
    assert_eq!(light.empty, [196.0, 200.0, 206.0]);
    assert_eq!(light.ctx_from, [61.0, 139.0, 235.0]);
    assert!(luminance(dark.empty) < luminance(dark.ctx_to));
    assert!(luminance(light.empty) > luminance(light.ctx_from));
    assert!(luminance(light.empty) > luminance(light.ctx_to));
}

#[test]
fn the_small_glyph_marks_a_real_half_cell_only() {
    let dots = "\"glyphs\": {\"empty\": \".\"}";
    let bar = |pct| glyphs(&ctx_only(pct, dots));
    assert_eq!(
        bar(44.0),
        "ctx \u{25A0}\u{25A0}\u{25A0}\u{25AA}.... 440k/1M\n"
    );
    assert_eq!(
        bar(50.0),
        "ctx \u{25A0}\u{25A0}\u{25A0}\u{25A0}.... 500k/1M\n"
    );
    assert_eq!(
        bar(100.0),
        "ctx \u{25A0}\u{25A0}\u{25A0}\u{25A0}\u{25A0}\u{25A0}\u{25A0}\u{25A0} 1M/1M\n"
    );
    assert_eq!(bar(6.0), "ctx \u{25AA}....... 60k/1M\n");
}

#[test]
fn the_context_bar_turns_warm_from_70_and_hot_from_90() {
    let first_cell = |pct: f64, config: &str| {
        let text = ctx_only(pct, config);
        let at = text.find("[#").unwrap();
        String::from(&text[at..at + 9])
    };
    assert_eq!(first_cell(69.0, "\"bold\": true"), "[#96cdff]");
    assert_eq!(first_cell(70.0, "\"bold\": true"), "[#be823c]");
    assert_eq!(first_cell(89.0, "\"bold\": true"), "[#be823c]");
    assert_eq!(first_cell(90.0, "\"bold\": true"), "[#c83c3c]");
    let raised = "\"thresholds\": {\"ctx_warn\": 50, \"ctx_hot\": 60}";
    assert_eq!(first_cell(49.0, raised), "[#96cdff]");
    assert_eq!(first_cell(50.0, raised), "[#be823c]");
    assert_eq!(first_cell(60.0, raised), "[#c83c3c]");
}

#[test]
fn the_cpu_bar_is_light_enough_to_see_on_dark() {
    assert_eq!(Config::parse(None).pal_cpu.ok, [79.0, 127.0, 196.0]);
    let text = line(false, None, &sample());
    assert!(text.contains("cpu[/] [#4f7fc4]"), "{}", text);
}

#[test]
fn the_auto_compact_marker_is_dimmed_against_ctx() {
    let text = line(false, None, &sample());
    assert!(text.starts_with("[dim]\u{AA}[/][b]ctx[/]"), "{}", text);
    let custom = line(false, Some("{\"auto_compact_marker\": \"~\"}"), &sample());
    assert!(custom.starts_with("[dim]~[/][b]ctx[/]"), "{}", custom);
    let none = line(false, Some("{\"auto_compact_marker\": \"\"}"), &sample());
    assert!(none.starts_with("[b]ctx[/]"), "{}", none);
}
