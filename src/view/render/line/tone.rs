use crate::config::Rgb;

pub(super) type Pair = (Rgb, Rgb);

#[derive(Clone, Copy)]
pub(super) enum Tone {
    Low,
    Medium,
    High,
    ExtraHigh,
    Max,
    Beyond,
}

const TONES: usize = Tone::Beyond as usize + 1;

const LIGHT_TONES: [Pair; TONES] = [
    ([70.0, 100.0, 97.0], [72.0, 92.0, 114.0]),
    ([71.0, 90.0, 115.0], [77.0, 85.0, 117.0]),
    ([75.0, 84.0, 118.0], [90.0, 81.0, 117.0]),
    ([88.0, 80.0, 118.0], [102.0, 78.0, 116.0]),
    ([102.0, 78.0, 116.0], [114.0, 80.0, 114.0]),
    ([116.0, 80.0, 117.0], [128.0, 93.0, 121.0]),
];

const DARK_TONES: [Pair; TONES] = [
    ([120.0, 172.0, 168.0], [124.0, 158.0, 196.0]),
    ([122.0, 156.0, 198.0], [132.0, 146.0, 202.0]),
    ([130.0, 144.0, 204.0], [156.0, 140.0, 202.0]),
    ([152.0, 138.0, 204.0], [176.0, 134.0, 200.0]),
    ([176.0, 134.0, 200.0], [196.0, 138.0, 196.0]),
    ([200.0, 138.0, 202.0], [220.0, 160.0, 208.0]),
];

impl Tone {
    pub(super) fn of(effort: &str) -> Tone {
        match effort {
            "low" | "auto" => Tone::Low,
            "medium" => Tone::Medium,
            "high" => Tone::High,
            "xhigh" => Tone::ExtraHigh,
            "max" => Tone::Max,
            _ => Tone::Beyond,
        }
    }

    pub(super) fn pair(self, light: bool) -> Pair {
        let tones = if light { &LIGHT_TONES } else { &DARK_TONES };
        tones[self as usize]
    }
}
