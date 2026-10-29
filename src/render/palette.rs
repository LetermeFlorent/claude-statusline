use crate::config::Rgb;

const CUBE_LEVELS: [i64; 6] = [0, 95, 135, 175, 215, 255];
const CUBE_BASE: i64 = 16;
const CUBE_SIDE: i64 = 6;
const GRAY_BASE: i64 = 232;
const GRAY_FIRST: i64 = 8;
const GRAY_STEP: i64 = 10;
const GRAY_COUNT: i64 = 24;

fn channel(value: f64) -> i64 {
    (value as i64).clamp(0, 255)
}

fn nearest_level(value: i64) -> i64 {
    (0..CUBE_SIDE)
        .min_by_key(|level| (CUBE_LEVELS[*level as usize] - value).abs())
        .unwrap_or(0)
}

fn distance(a: [i64; 3], b: [i64; 3]) -> i64 {
    (0..3).map(|i| (a[i] - b[i]) * (a[i] - b[i])).sum()
}

pub fn indexed(color: Rgb) -> u8 {
    let wanted = color.map(channel);
    let levels = wanted.map(nearest_level);
    let cube = levels.map(|level| CUBE_LEVELS[level as usize]);
    let cube_index =
        CUBE_BASE + CUBE_SIDE * CUBE_SIDE * levels[0] + CUBE_SIDE * levels[1] + levels[2];
    let mean = (wanted[0] + wanted[1] + wanted[2]) / 3;
    let step = ((mean - GRAY_FIRST + GRAY_STEP / 2) / GRAY_STEP).clamp(0, GRAY_COUNT - 1);
    let gray = GRAY_FIRST + GRAY_STEP * step;
    if distance(wanted, [gray; 3]) < distance(wanted, cube) {
        (GRAY_BASE + step) as u8
    } else {
        cube_index as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_map_to_the_nearest_of_the_256_palette() {
        assert_eq!(indexed([0.0, 0.0, 0.0]), 16);
        assert_eq!(indexed([255.0, 255.0, 255.0]), 231);
        assert_eq!(indexed([255.0, 0.0, 0.0]), 196);
        assert_eq!(indexed([0.0, 0.0, 255.0]), 21);
        assert_eq!(indexed([128.0, 128.0, 128.0]), 244);
        assert_eq!(indexed([85.0, 89.0, 95.0]), 240);
        assert_eq!(indexed([300.0, -4.0, 0.0]), 196);
    }
}
