use super::hex::hex;
use super::{Palette, Rgb};
use crate::json::Value;
use alloc::vec::Vec;

pub const WARN_SYS: Rgb = [190.0, 130.0, 60.0];
pub const HOT_SYS: Rgb = [200.0, 60.0, 60.0];

pub(super) struct PaletteSpec {
    path: &'static [&'static str],
    ok: Rgb,
    warn: Rgb,
    hot: Rgb,
}

pub(super) const PAL_5H: PaletteSpec = PaletteSpec {
    path: &["palettes", "5h"],
    ok: [120.0, 110.0, 200.0],
    warn: [170.0, 90.0, 175.0],
    hot: [200.0, 60.0, 95.0],
};
pub(super) const PAL_7D: PaletteSpec = PaletteSpec {
    path: &["palettes", "7d"],
    ok: [83.0, 137.0, 119.0],
    warn: [184.0, 129.0, 67.0],
    hot: [185.0, 85.0, 85.0],
};
pub(super) const PAL_RAM: PaletteSpec = PaletteSpec {
    path: &["system", "colors", "ram"],
    ok: [30.0, 120.0, 125.0],
    warn: WARN_SYS,
    hot: HOT_SYS,
};
pub(super) const PAL_DSK: PaletteSpec = PaletteSpec {
    path: &["system", "colors", "dsk"],
    ok: [105.0, 120.0, 85.0],
    warn: WARN_SYS,
    hot: HOT_SYS,
};
pub(super) const PAL_CPU: PaletteSpec = PaletteSpec {
    path: &["system", "colors", "cpu"],
    ok: [79.0, 127.0, 196.0],
    warn: WARN_SYS,
    hot: HOT_SYS,
};

pub(super) fn palette(raw: &Value, spec: &PaletteSpec) -> Palette {
    let get = |k: &str, def: Rgb| {
        let mut p: Vec<&str> = spec.path.to_vec();
        p.push(k);
        hex(raw.at(&p)).unwrap_or(def)
    };
    Palette {
        ok: get("ok", spec.ok),
        warn: get("warn", spec.warn),
        hot: get("hot", spec.hot),
    }
}
