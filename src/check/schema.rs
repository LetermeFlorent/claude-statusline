use crate::config::{by_name, hex, BACKGROUND_NAMES, EFFORT_NAMES, METRIC_NAMES};
use crate::json::Value;

#[derive(Debug, PartialEq)]
pub enum Kind {
    Flag,
    Number,
    Text,
    Color,
    Background,
    Effort,
    Metric,
    List(&'static Kind),
    Object(Keys),
}

pub type Keys = &'static [(&'static str, Kind)];

const PALETTE: Kind = Kind::Object(&[
    ("ok", Kind::Color),
    ("warn", Kind::Color),
    ("hot", Kind::Color),
]);

pub const SETTINGS: Keys = &[
    ("terminal_background", Kind::Background),
    ("bar_width", Kind::Number),
    ("bold", Kind::Flag),
    (
        "glyphs",
        Kind::Object(&[
            ("filled", Kind::Text),
            ("empty", Kind::Text),
            ("half", Kind::Text),
        ]),
    ),
    ("auto_compact_marker", Kind::Text),
    ("effort_style", Kind::Effort),
    (
        "segments",
        Kind::Object(&[
            ("ctx", Kind::Flag),
            ("5h", Kind::Flag),
            ("7d", Kind::Flag),
            ("spend", Kind::Flag),
            ("model", Kind::Flag),
        ]),
    ),
    (
        "gradient",
        Kind::Object(&[("5h", Kind::Number), ("7d", Kind::Number)]),
    ),
    (
        "thresholds",
        Kind::Object(&[
            ("ctx_warn", Kind::Number),
            ("ctx_hot", Kind::Number),
            ("quota_warn", Kind::Number),
            ("quota_hot", Kind::Number),
            ("system_warn", Kind::Number),
            ("system_hot", Kind::Number),
        ]),
    ),
    (
        "palettes",
        Kind::Object(&[("5h", PALETTE), ("7d", PALETTE)]),
    ),
    (
        "colors",
        Kind::Object(&[
            ("ink_light", Kind::Color),
            ("ink_dark", Kind::Color),
            ("empty_light", Kind::Color),
            ("empty_dark", Kind::Color),
            ("ctx_from_light", Kind::Color),
            ("ctx_from_dark", Kind::Color),
            ("ctx_to_light", Kind::Color),
            ("ctx_to_dark", Kind::Color),
        ]),
    ),
    (
        "system",
        Kind::Object(&[
            ("enabled", Kind::Flag),
            ("metrics", Kind::List(&Kind::Metric)),
            ("gradient", Kind::Number),
            ("disk_path", Kind::Text),
            (
                "colors",
                Kind::Object(&[("ram", PALETTE), ("dsk", PALETTE), ("cpu", PALETTE)]),
            ),
        ]),
    ),
    ("auto_update", Kind::Flag),
];

impl Kind {
    pub fn accepts(&self, value: &Value) -> bool {
        match self {
            Kind::Flag => value.bool().is_some(),
            Kind::Number => value.num().is_some(),
            Kind::Text => value.str().is_some(),
            Kind::Color => hex(Some(value)).is_some(),
            Kind::Background => value
                .str()
                .and_then(|s| by_name(&BACKGROUND_NAMES, s))
                .is_some(),
            Kind::Effort => value
                .str()
                .and_then(|s| by_name(&EFFORT_NAMES, s))
                .is_some(),
            Kind::Metric => value
                .str()
                .and_then(|s| by_name(&METRIC_NAMES, s))
                .is_some(),
            Kind::List(_) => value.arr().is_some(),
            Kind::Object(_) => value.is_obj(),
        }
    }
}
