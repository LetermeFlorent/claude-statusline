use super::common::{line, sample, DISK};
use alloc::format;

#[test]
fn dark_line() {
    let expected = format!(
        concat!(
            "[dim]ª[/][b]ctx[/] [#96cdff]━━━━━[#55595f]╌╌╌╌╌╌╌╌╌╌╌[/] [b]60k/200k[/]",
            " [b]|[/] [b]5h[/]  [#aa5aaf]━━━━━━[#55595f]╌╌╌╌╌╌╌╌╌╌[/]    [b]36%[/] [b]1h00m[/]",
            " [b]|[/] [b]7d[/]  [#b95555]━━━━━━━━━━━━━[#55595f]╌╌╌[/]    [b]81%[/] [b]2d07h[/]",
            " [b]|[/] Opus 4[/][b]:[/][b][#f59b46]h[b][#f39143]i[b][#f2873f]g[b][#f07d3c]h[/]\n",
            "[b]ram[/] [#be823c]━━━━━━━━━━━━[#55595f]╌╌╌╌[/] [b]12.0/16.0G[/]",
            " [b]|[/] [b]{disk}[/] [#c83c3c]━━━━━━━━━━━━━━[#55595f]╌╌[/]  [b]90%[/]",
            " [b]|[/] [b]cpu[/] [#4f7fc4]━━[#55595f]╌╌╌╌╌╌╌╌╌╌╌╌╌╌[/]  [b]12%[/]",
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
            "[dim]ª[/][b]ctx[/] [#3d8beb]━━━━━[#c4c8ce]╌╌╌╌╌╌╌╌╌╌╌[/] [b]60k/200k[/]",
            " [b]|[/] [b]5h[/]  [#aa5aaf]━━━━━━[#c4c8ce]╌╌╌╌╌╌╌╌╌╌[/]    [b]36%[/] [b]1h00m[/]",
            " [b]|[/] [b]7d[/]  [#b95555]━━━━━━━━━━━━━[#c4c8ce]╌╌╌[/]    [b]81%[/] [b]2d07h[/]",
            " [b]|[/] Opus 4[/][b]:[/][b][#c85a00]h[b][#c55303]i[b][#c14d07]g[b][#be460a]h[/]\n",
            "[b]ram[/] [#be823c]━━━━━━━━━━━━[#c4c8ce]╌╌╌╌[/] [b]12.0/16.0G[/]",
            " [b]|[/] [b]{disk}[/] [#c83c3c]━━━━━━━━━━━━━━[#c4c8ce]╌╌[/]  [b]90%[/]",
            " [b]|[/] [b]cpu[/] [#4f7fc4]━━[#c4c8ce]╌╌╌╌╌╌╌╌╌╌╌╌╌╌[/]  [b]12%[/]",
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
        assert!(text.contains("Opus 4[/][b]:[/]"), "{}", text);
    }
}

#[test]
fn ink_colors_the_text_when_it_is_set() {
    let config = "{\"colors\": {\"ink_dark\": \"#112233\"}}";
    let text = line(false, Some(config), &sample());
    assert!(text.contains("[b][#112233]ctx[/]"), "{}", text);
    assert!(text.contains("[b][#112233]60k/200k[/]"), "{}", text);
    assert!(text.contains("[#112233]Opus 4[/]"), "{}", text);
    let light = line(true, Some(config), &sample());
    assert!(light.contains("[b]ctx[/]"), "{}", light);
}
