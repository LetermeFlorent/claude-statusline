use crate::num;

pub fn cells(pct: f64, width: usize) -> usize {
    if !(pct > 0.0) {
        return 0;
    }
    (num::round(pct * width as f64 / 100.0) as usize).clamp(1, width)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bar_at_100_percent_or_more_is_full() {
        assert_eq!(cells(100.0, 16), 16);
        assert_eq!(cells(100.0, 1), 1);
        assert_eq!(cells(150.0, 16), 16);
    }

    #[test]
    fn the_filled_cells_are_the_value_rounded_to_a_cell() {
        assert_eq!(cells(50.0, 16), 8);
        assert_eq!(cells(52.3, 16), 8);
        assert_eq!(cells(56.0, 16), 9);
        assert_eq!(cells(25.0, 8), 2);
    }

    #[test]
    fn any_value_above_zero_fills_one_cell_at_least() {
        assert_eq!(cells(0.5, 16), 1);
        assert_eq!(cells(0.0, 16), 0);
        assert_eq!(cells(-4.0, 16), 0);
        assert_eq!(cells(f64::NAN, 16), 0);
    }
}
