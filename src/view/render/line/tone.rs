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
    ([46.0, 125.0, 50.0], [56.0, 142.0, 60.0]),
    ([160.0, 115.0, 0.0], [175.0, 100.0, 0.0]),
    ([200.0, 90.0, 0.0], [190.0, 70.0, 10.0]),
    ([198.0, 40.0, 40.0], [180.0, 30.0, 70.0]),
    ([156.0, 39.0, 176.0], [120.0, 50.0, 190.0]),
    ([0.0, 121.0, 140.0], [21.0, 101.0, 192.0]),
];

const DARK_TONES: [Pair; TONES] = [
    ([102.0, 187.0, 106.0], [129.0, 199.0, 132.0]),
    ([230.0, 200.0, 90.0], [240.0, 185.0, 70.0]),
    ([245.0, 155.0, 70.0], [240.0, 125.0, 60.0]),
    ([240.0, 95.0, 90.0], [225.0, 75.0, 105.0]),
    ([215.0, 100.0, 200.0], [185.0, 110.0, 235.0]),
    ([80.0, 200.0, 215.0], [100.0, 170.0, 240.0]),
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
