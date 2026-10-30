use super::size;
use crate::json::{self, Syntax, Value};
use alloc::format;
use alloc::string::String;

const OFF: &str = "off";

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Setting {
    Off,
    Window(u64),
}

impl Setting {
    pub fn store(self) -> String {
        match self {
            Setting::Off => String::from(OFF),
            Setting::Window(n) => format!("{}", n),
        }
    }

    pub fn load(text: &str) -> Option<Setting> {
        if text == OFF {
            return Some(Setting::Off);
        }
        text.parse::<u64>()
            .ok()
            .filter(|n| *n > 0)
            .map(Setting::Window)
    }
}

#[derive(Default)]
struct Found {
    enabled: Option<bool>,
    model: Option<u64>,
    env: Option<u64>,
    plain: Option<u64>,
}

impl Found {
    fn merge(&mut self, settings: &Value, model: Option<&str>) {
        let window = |path: &[&str]| settings.at(path).and_then(size::from_value);
        self.enabled = settings
            .get("autoCompactEnabled")
            .and_then(|x| x.bool())
            .or(self.enabled);
        if let Some(model) = model {
            let path = ["modelSettings", model, "autoCompactWindow"];
            self.model = window(&path).or(self.model);
        }
        let env = ["env", "CLAUDE_CODE_AUTO_COMPACT_WINDOW"];
        self.env = window(&env).or(self.env);
        self.plain = window(&["autoCompactWindow"]).or(self.plain);
    }

    fn resolve(&self) -> Option<Setting> {
        if self.enabled == Some(false) {
            return Some(Setting::Off);
        }
        self.model
            .or(self.env)
            .or(self.plain)
            .map(|n| Setting::Window(size::clamp(n)))
    }
}

pub fn scan(
    files: &[String],
    model: Option<&str>,
    read: &dyn Fn(&str) -> Option<String>,
) -> Option<Setting> {
    let mut found = Found::default();
    for file in files {
        let settings = read(file)
            .and_then(|t| json::parse(&t, Syntax::Commented))
            .filter(|v| v.is_obj());
        if let Some(settings) = settings {
            found.merge(&settings, model);
        }
    }
    found.resolve()
}
