use super::{j, none};
use crate::theme::hosts::color::color_mode;
use crate::theme::hosts::ghostty::{ghostty, ghostty_keys};
use crate::theme::hosts::vscode::vscode;
use crate::theme::hosts::wt::wt;
use crate::theme::hosts::zed::zed;
use crate::theme::{colorfgbg, Mode};

#[test]
fn terminal_settings_map_to_light_dark_or_system() {
    assert_eq!(colorfgbg("0;15"), Some(true));
    assert_eq!(colorfgbg("15;0"), Some(false));
    assert_eq!(colorfgbg("15;default;7"), Some(true));
    assert_eq!(colorfgbg("0;-1"), None);
    assert_eq!(
        vscode(
            &[j("{\"workbench.colorTheme\": \"Default Light Modern\"}")],
            &none
        ),
        Mode::Light
    );
    assert_eq!(
        vscode(
            &[j("{\"workbench\": {\"colorTheme\": \"Monokai\"}}")],
            &none
        ),
        Mode::Dark
    );
    assert_eq!(
        vscode(
            &[j(
                "{\"window.autoDetectColorScheme\": true, \"workbench.colorTheme\": \"x\"}"
            )],
            &none
        ),
        Mode::System
    );
    assert_eq!(vscode(&[j("{}")], &none), Mode::Dark);
    let dark = || false;
    let light = || true;
    let s = "{\"profiles\": {\"defaults\": {\"colorScheme\": \"Mine\"}, \"list\": [{\"guid\": \"{A}\", \"colorScheme\": \"Tango Light\"}]},
             \"schemes\": [{\"name\": \"Mine\", \"background\": \"#FAFAFA\"}, {\"name\": \"Tango Light\", \"background\": \"#000\"}],}";
    assert_eq!(wt(&j(s), None, &dark), Mode::Light);
    assert_eq!(wt(&j(s), Some("{a}"), &dark), Mode::Dark);
    assert_eq!(wt(&j("{}"), None, &dark), Mode::Dark);
    assert_eq!(wt(&j("{\"profiles\": [{\"guid\": \"g\", \"colorScheme\": \"One Half Light\"}], \"defaultProfile\": \"g\"}"), None, &dark), Mode::Light);
    let pair = "{\"profiles\": {\"defaults\": {\"colorScheme\": {\"light\": \"Solarized Light\", \"dark\": \"Campbell\"}}}}";
    assert_eq!(wt(&j(pair), None, &light), Mode::Light);
    assert_eq!(wt(&j(pair), None, &dark), Mode::Dark);
    let legacy = pair.replacen('{', "{\"theme\": \"legacyLight\", ", 1);
    assert_eq!(wt(&j(&legacy), None, &dark), Mode::Light);
    assert_eq!(
        wt(
            &j("{\"profiles\": {\"defaults\": {\"background\": \"#ffffff\"}}}"),
            None,
            &dark
        ),
        Mode::Light
    );
    assert_eq!(
        zed(&j("{\"theme\": {\"mode\": \"system\"}}"), &none),
        Mode::System
    );
    assert_eq!(zed(&j("{\"theme\": \"One Light\"}"), &none), Mode::Light);
    assert_eq!(
        ghostty("theme = light:Builtin Light,dark:Builtin Dark\n", &none),
        Mode::System
    );
    assert_eq!(
        ghostty("background = #fdf6e3\ntheme = Dracula\n", &none),
        Mode::Light
    );
    assert_eq!(ghostty("", &none), Mode::Dark);
}

#[test]
fn vscode_reads_the_terminal_background_then_the_workspace_then_the_theme_file() {
    let scoped = r##"{"workbench.colorTheme": "Solarized Light", "workbench.colorCustomizations":
        {"[Solarized Light]": {"terminal.background": "#002b36"}, "terminal.background": "#ffffff"}}"##;
    assert_eq!(vscode(&[j(scoped)], &none), Mode::Dark);
    let workspace = j(r#"{"workbench.colorTheme": "Abyss"}"#);
    let user = j(r#"{"workbench.colorTheme": "Quiet Light"}"#);
    assert_eq!(vscode(&[workspace, user], &none), Mode::Dark);
    let alabaster = |name: &str| (name == "Alabaster").then_some(Mode::Light);
    let custom = j(r#"{"workbench.colorTheme": "Alabaster"}"#);
    assert_eq!(vscode(&[custom], &alabaster), Mode::Light);
}

#[test]
fn colors_and_ghostty_keys_follow_the_last_valid_value() {
    assert_eq!(color_mode("#fdf6e3cc"), Some(Mode::Light));
    assert_eq!(color_mode("#fffa"), Some(Mode::Light));
    assert_eq!(color_mode("282c34"), Some(Mode::Dark));
    assert_eq!(color_mode("#12345"), None);
    assert_eq!(color_mode("abcde\u{e9}x"), None);
    let kept = ghostty_keys("background = #ffffff\nbackground = nope\n");
    assert_eq!(kept.0, Some(Mode::Light));
    assert_eq!(ghostty_keys("background = #ffffff\nbackground =\n").0, None);
    assert_eq!(ghostty_keys("theme = Dracula\ntheme = \"\"\n").1, None);
    assert_eq!(ghostty_keys("theme = Dracula\n").1, Some("Dracula"));
}
