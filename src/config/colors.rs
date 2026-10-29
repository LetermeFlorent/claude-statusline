use super::hex::hex;
use super::{Colors, Config, Rgb};

impl Config {
    pub fn colors(&self, light: bool) -> Colors {
        let t = if light { "light" } else { "dark" };
        let custom = |k: &str| {
            let key = alloc::format!("{}_{}", k, t);
            hex(self.raw.at(&["colors", &key]))
        };
        let color = |k: &str, def: Rgb| custom(k).unwrap_or(def);
        if light {
            Colors {
                light,
                ink: custom("ink"),
                empty: color("empty", [196.0, 200.0, 206.0]),
                ctx_from: color("ctx_from", [61.0, 139.0, 235.0]),
                ctx_to: color("ctx_to", [40.0, 90.0, 180.0]),
            }
        } else {
            Colors {
                light,
                ink: custom("ink"),
                empty: color("empty", [85.0, 89.0, 95.0]),
                ctx_from: color("ctx_from", [150.0, 205.0, 255.0]),
                ctx_to: color("ctx_to", [70.0, 130.0, 220.0]),
            }
        }
    }
}
