mod segments;

use super::*;

#[test]
fn labels_shorten_and_cells_stay_within_the_bar() {
    assert_eq!(tokens(2500), "3k");
    assert_eq!(tokens(3500), "4k");
    assert_eq!(tokens(999), "999");
    assert_eq!(reset_text(59.9, false), "0m");
    assert_eq!(reset_text(5000.0, false), "1h23m");
    assert_eq!(reset_text(200000.0, true), "2d07h");
    assert_eq!(reset_text(-5.0, true), "0m");
}

#[test]
fn huge_values_are_clamped() {
    assert_eq!(tokens(u64::MAX), "999B");
    assert_eq!(reset_text(1e300, true), "99d00h");
    assert_eq!(reset_text(f64::INFINITY, false), "2376h00m");
    assert_eq!(reset_text(f64::NAN, true), "0m");
}

const NOW: f64 = 1_000_000.0;

fn readable(rendered: &str) -> String {
    let mut parts = rendered.split('\x1b');
    let mut out = String::from(parts.next().unwrap_or(""));
    for part in parts {
        let (code, text) = part[1..].split_once('m').unwrap();
        match code {
            "0" => out.push_str("[/]"),
            "1" => out.push_str("[b]"),
            "2" => out.push_str("[dim]"),
            "4" => out.push_str("[u]"),
            _ => {
                let channels: Vec<u8> = code
                    .strip_prefix("38;2;")
                    .unwrap()
                    .split(';')
                    .map(|x| x.parse().unwrap())
                    .collect();
                let _ = write!(
                    out,
                    "[#{:02x}{:02x}{:02x}]",
                    channels[0], channels[1], channels[2]
                );
            }
        }
        out.push_str(text);
    }
    out
}

fn sample() -> Data {
    Data {
        ctx: Ctx {
            size: Some(1_000_000.0),
            pct: Some(6.0),
            tokens: Some(60_000),
        },
        window: Some(200_000),
        quotas: Some(Quotas {
            five_hour: Some(Quota {
                pct: Some(36.0),
                at: Some(NOW + 3600.0),
            }),
            seven_day: Some(Quota {
                pct: Some(81.0),
                at: Some(NOW + 200_000.0),
            }),
        }),
        spend: None,
        cache_warm: None,
        model: String::from("Opus 4"),
        effort: Some(String::from("high")),
        sys: alloc::vec![
            SysValue {
                metric: Metric::Ram,
                pct: None,
                ram: Some((16.0 * GIB, 4.0 * GIB)),
            },
            SysValue {
                metric: Metric::Io,
                pct: Some(90.0),
                ram: None,
            },
            SysValue {
                metric: Metric::Cpu,
                pct: Some(12.0),
                ram: None,
            },
        ],
        updating: true,
    }
}

fn line(light: bool, config: Option<&str>, data: &Data) -> String {
    let cfg = Config::parse(config);
    readable(&render(
        &cfg,
        &cfg.colors(light),
        data,
        NOW,
        &Terminal::WIDE,
    ))
}

fn fitted(columns: usize, config: Option<&str>) -> String {
    let cfg = Config::parse(config);
    let term = Terminal {
        columns: Some(columns),
        depth: Depth::TrueColor,
    };
    readable(&render(&cfg, &cfg.colors(false), &sample(), NOW, &term))
}

fn quota_only(pct: Option<f64>) -> String {
    let mut data = sample();
    data.quotas = Some(Quotas {
        five_hour: Some(Quota { pct, at: None }),
        seven_day: None,
    });
    data.sys.clear();
    data.updating = false;
    let only_5h = "{\"segments\": {\"ctx\": false, \"7d\": false, \"model\": false}}";
    line(false, Some(only_5h), &data)
}

const DISK: &str = if cfg!(windows) { "C:" } else { "dsk" };

#[test]
fn dark_line() {
    let expected = format!(
        concat!(
            "[b]~[/] [b]ctx[/] [#96cdff]■[#8bc2fa]■[#7fb8f5]▪[#55595f]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[/] [b]60k/200k[/]",
            " [b]|[/] [b]5h[/] [#aa5aaf]■[#b168b6]■[#b876bc]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[/] [b]36%[/] [b]1h00m[/]",
            " [b]|[/] [b]7d[/] [#b95555]■[#ba5858]■[#bb5a5a]■[#bc5d5d]■[#bd5f5f]■[#be6262]■[#bf6464]▪[#55595f]■[/] [b]81%[/] [b]2d07h[/]",
            " [b]|[/] [u]Opus 4[/][b]:[/][b][#8290cc]h[b][#8b8fcb]i[b][#938dcb]g[b][#9c8cca]h[/]\n",
            "[b]ram[/] [#be823c]■[#c08541]■[#c28947]■[#c38c4c]■[#c59051]■[#c79357]■[#55595f]■[#55595f]■[/] [b]12.0/16.0G[/]",
            " [b]|[/] [b]{disk}[/] [#c83c3c]■[#c94040]■[#cb4545]■[#cc4949]■[#cd4e4e]■[#ce5252]■[#d05757]■[#55595f]■[/] [b]90%[/]",
            " [b]|[/] [b]cpu[/] [#4f7fc4]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[#55595f]■[/] [b]12%[/]",
            " [b]|[/] [b]update[/][/]\n",
        ),
        disk = DISK
    );
    assert_eq!(line(false, None, &sample()), expected);
}

#[test]
fn light_line() {
    let expected = format!(
        concat!(
            "[b]~[/] [b]ctx[/] [#3d8beb]■[#3a84e3]■[#377ddb]▪[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[/] [b]60k/200k[/]",
            " [b]|[/] [b]5h[/] [#aa5aaf]■[#9a519e]■[#8a498d]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[/] [b]36%[/] [b]1h00m[/]",
            " [b]|[/] [b]7d[/] [#b95555]■[#b65454]■[#b35252]■[#af5151]■[#ac4f4f]■[#a94e4e]■[#a64c4c]▪[#c4c8ce]■[/] [b]81%[/] [b]2d07h[/]",
            " [b]|[/] [u]Opus 4[/][b]:[/][b][#4b5476]h[b][#505376]i[b][#555275]g[b][#5a5175]h[/]\n",
            "[b]ram[/] [#be823c]■[#b87e3a]■[#b27a38]■[#ac7636]■[#a67234]■[#a06e33]■[#c4c8ce]■[#c4c8ce]■[/] [b]12.0/16.0G[/]",
            " [b]|[/] [b]{disk}[/] [#c83c3c]■[#c33a3a]■[#be3939]■[#b83737]■[#b33636]■[#ae3434]■[#a93333]■[#c4c8ce]■[/] [b]90%[/]",
            " [b]|[/] [b]cpu[/] [#4f7fc4]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[#c4c8ce]■[/] [b]12%[/]",
            " [b]|[/] [b]update[/][/]\n",
        ),
        disk = DISK
    );
    assert_eq!(line(true, None, &sample()), expected);
}

#[test]
fn text_keeps_the_terminal_color_without_ink() {
    for light in [false, true] {
        let text = line(light, None, &sample());
        for word in [
            "ctx", "60k/200k", "|", "5h", "36%", "1h00m", "ram", "update",
        ] {
            assert!(
                text.contains(&format!("[b]{}[/]", word)),
                "{} {}",
                word,
                text
            );
        }
        assert!(text.contains("[u]Opus 4[/][b]:[/]"), "{}", text);
    }
}

#[test]
fn ink_colors_the_text_when_it_is_set() {
    let config = "{\"colors\": {\"ink_dark\": \"#112233\"}}";
    let text = line(false, Some(config), &sample());
    assert!(text.contains("[b][#112233]ctx[/]"), "{}", text);
    assert!(text.contains("[b][#112233]60k/200k[/]"), "{}", text);
    assert!(text.contains("[u][#112233]Opus 4[/]"), "{}", text);
    let light = line(true, Some(config), &sample());
    assert!(light.contains("[b]ctx[/]"), "{}", light);
}

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

fn luminance(c: Rgb) -> f64 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
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

fn ctx_only(pct: f64, config: &str) -> String {
    let mut data = sample();
    data.ctx = Ctx {
        size: Some(1_000_000.0),
        pct: Some(pct),
        tokens: Some((pct * 10_000.0) as u64),
    };
    data.window = None;
    data.quotas = None;
    data.sys.clear();
    data.updating = false;
    let only_ctx = "\"segments\": {\"5h\": false, \"7d\": false, \"model\": false}";
    let text = format!("{{{}, {}}}", only_ctx, config);
    line(false, Some(&text), &data)
}

fn glyphs(rendered: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for c in rendered.chars() {
        match c {
            '[' => inside = true,
            ']' => inside = false,
            c if !inside => out.push(c),
            _ => {}
        }
    }
    out
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
fn the_auto_compact_marker_is_a_visible_tilde_and_a_space() {
    let text = line(false, None, &sample());
    assert!(text.starts_with("[b]~[/] [b]ctx[/]"), "{}", text);
    let old = line(
        false,
        Some("{\"auto_compact_marker\": \"\u{AA}\"}"),
        &sample(),
    );
    assert!(old.starts_with("[b]\u{AA}[/] [b]ctx[/]"), "{}", old);
    let none = line(false, Some("{\"auto_compact_marker\": \"\"}"), &sample());
    assert!(none.starts_with("[b]ctx[/]"), "{}", none);
    assert!(!text.contains("[dim]"), "{}", text);
}

#[test]
fn a_count_close_to_a_million_moves_to_the_next_unit() {
    assert_eq!(tokens(999_499), "999k");
    assert_eq!(tokens(999_500), "1M");
    assert_eq!(tokens(999_999), "1M");
    assert_eq!(tokens(999_499_999), "999M");
    assert_eq!(tokens(999_500_000), "1B");
    assert_eq!(tokens(1_499), "1k");
    assert_eq!(tokens(1_500), "2k");
    assert_eq!(tokens(2_500), "3k");
}

#[test]
fn the_line_fits_in_80_columns() {
    let wide = glyphs(&line(false, None, &sample()));
    assert!(fit::widest(&wide) > 80, "{}", wide);
    let narrow = glyphs(&fitted(80, None));
    assert!(fit::widest(&narrow) <= 80, "{}", narrow);
}

#[test]
fn the_reset_times_go_first_then_the_effort_then_the_bar_width() {
    let text = |columns| glyphs(&fitted(columns, None));
    let bar = "\u{25A0}".repeat(8);
    assert!(text(85).contains("1h00"), "{}", text(85));
    let no_resets = text(84);
    assert!(!no_resets.contains("1h00"), "{}", no_resets);
    assert!(no_resets.contains(":high"), "{}", no_resets);
    let no_effort = text(70);
    assert!(!no_effort.contains(":high"), "{}", no_effort);
    assert!(no_effort.contains("Opus 4"), "{}", no_effort);
    assert!(no_effort.contains(&bar), "{}", no_effort);
    let narrow = text(60);
    assert!(!narrow.contains(&bar), "{}", narrow);
    assert!(fit::widest(&narrow) <= 60, "{}", narrow);
}

#[test]
fn a_bar_width_of_4_fits_in_80_columns_with_every_detail() {
    let text = glyphs(&fitted(80, Some("{\"bar_width\": 4}")));
    assert!(text.contains("1h00") && text.contains(":high"), "{}", text);
    assert!(fit::widest(&text) <= 80, "{}", text);
}

fn raw(depth: Depth) -> String {
    let cfg = Config::parse(None);
    let term = Terminal {
        columns: None,
        depth,
    };
    render(&cfg, &cfg.colors(false), &sample(), NOW, &term)
}

#[test]
fn without_truecolor_only_256_color_sequences_are_sent() {
    let indexed = raw(Depth::Indexed);
    assert!(!indexed.contains("38;2;"), "{:?}", indexed);
    assert!(indexed.contains("[38;5;240m"), "{:?}", indexed);
    let colors = indexed.matches("38;5;").count();
    assert_eq!(colors, indexed.matches("[38;").count());
    let true_color = raw(Depth::TrueColor);
    assert!(!true_color.contains("38;5;"), "{:?}", true_color);
    assert!(true_color.contains("38;2;85;89;95"), "{:?}", true_color);
}

#[test]
fn negative_percent_reads_zero() {
    assert_eq!(pct_text(Some(-3.0)), "0%");
}
