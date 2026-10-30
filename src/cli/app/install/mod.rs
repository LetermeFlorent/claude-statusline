mod edit;
mod repair;

pub(in crate::cli::app) use edit::{add_line, remove_line};
use repair::repair_globals;

use crate::app::paths::{config_path, settings_path};
use crate::app::FAILED_EXIT;
use crate::messages;
use crate::os;
use crate::portable;

pub(in crate::cli::app) const TEMPLATE: &str = "{
  \"bar_width\": 8,
  \"bold\": true,
  \"glyphs\": { \"filled\": \"\u{25A0}\", \"empty\": \"\u{25A0}\", \"half\": \"\u{25AA}\" },
  \"auto_compact_marker\": \"~\",
  \"effort_style\": \"gradient\",
  \"segments\": { \"ctx\": true, \"5h\": true, \"7d\": true, \"model\": true },
  \"gradient\": { \"5h\": 0.55, \"7d\": 0.3 },
  \"system\": {
    \"enabled\": true,
    \"metrics\": [\"ram\", \"io\", \"cpu\"],
    \"gradient\": 0.45
  }
}
";

pub(super) fn install() -> i32 {
    let Some(path) = settings_path() else {
        return os::fail(messages::NO_HOME.text(), FAILED_EXIT);
    };
    let Some(exe) = os::exe() else {
        return os::fail(messages::NO_EXE.text(), FAILED_EXIT);
    };
    let command = portable::command(&exe);
    match add_line(&path, &command) {
        Ok(told) => os::out(&told),
        Err(message) => return os::fail(&message, FAILED_EXIT),
    }
    repair_globals(&command);
    let conf = config_path();
    if let Some(c) = conf.filter(|c| !os::exists(c)) {
        let target = os::claude_dir()
            .map(|d| os::join(&d, &["statusline.json"]))
            .unwrap_or(c);
        if os::write_atomic(&target, TEMPLATE.as_bytes()) {
            os::out(&messages::CONFIG_WRITTEN.with(&[&target]));
        }
    }
    os::out(messages::RESTART.text());
    0
}

pub(super) fn uninstall() -> i32 {
    let Some(path) = settings_path() else {
        return os::fail(messages::NO_HOME.text(), FAILED_EXIT);
    };
    match remove_line(&path) {
        Ok(told) => {
            os::out(&told);
            0
        }
        Err(message) => os::fail(&message, FAILED_EXIT),
    }
}
