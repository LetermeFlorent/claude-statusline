use super::common::{line, sample, DISK};
use alloc::format;

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
