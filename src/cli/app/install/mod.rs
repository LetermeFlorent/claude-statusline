mod edit;
mod plugin;
mod repair;

pub(in crate::cli::app) use edit::{add_line, remove_line};
#[cfg(test)]
pub(in crate::cli::app) use plugin::{folder as plugin_folder, write as write_plugin};
use repair::repair_globals;

use crate::app::paths::{config_path, settings_path};
use crate::app::FAILED_EXIT;
use crate::messages;
use crate::os;
use crate::portable;

pub(in crate::cli::app) const TEMPLATE: &str = "{
  \"bar_width\": 16,
  \"bold\": true,
  \"glyphs\": { \"filled\": \"\u{2501}\", \"empty\": \"\u{254C}\" },
  \"auto_compact_marker\": \"\u{AA}\",
  \"effort_style\": \"gradient\",
  \"segments\": { \"ctx\": true, \"5h\": true, \"7d\": true, \"model\": true },
  \"system\": {
    \"enabled\": true,
    \"metrics\": [\"ram\", \"io\", \"cpu\"]
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
    let plugin = plugin::folder(&exe).filter(|d| plugin::write(d));
    match add_line(
        &path,
        &command,
        plugin.as_deref().map(portable::tilde_path).as_deref(),
    ) {
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
    let plugin = os::exe()
        .and_then(|e| plugin::folder(&e))
        .map(|d| portable::tilde_path(&d));
    match remove_line(&path, plugin.as_deref()) {
        Ok(told) => {
            os::out(&told);
            0
        }
        Err(message) => os::fail(&message, FAILED_EXIT),
    }
}
