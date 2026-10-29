use crate::num;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Fill {
    pub cells: usize,
    pub half: bool,
}

const EMPTY: Fill = Fill {
    cells: 0,
    half: false,
};

pub fn of(pct: f64, width: usize) -> Fill {
    if !(pct > 0.0) {
        return EMPTY;
    }
    let halves = num::round(pct * 2.0 * width as f64 / 100.0);
    if halves < 1.0 {
        return Fill {
            cells: 1,
            half: true,
        };
    }
    if halves >= 2.0 * width as f64 {
        return Fill {
            cells: width,
            half: false,
        };
    }
    Fill {
        cells: num::ceil(halves / 2.0) as usize,
        half: halves as usize % 2 == 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fill(cells: usize, half: bool) -> Fill {
        Fill { cells, half }
    }

    #[test]
    fn a_bar_at_100_percent_is_full_with_no_half_cell() {
        assert_eq!(of(100.0, 8), fill(8, false));
        assert_eq!(of(100.0, 1), fill(1, false));
        assert_eq!(of(150.0, 8), fill(8, false));
    }

    #[test]
    fn an_even_number_of_half_cells_ends_on_a_full_cell() {
        assert_eq!(of(50.0, 8), fill(4, false));
        assert_eq!(of(25.0, 8), fill(2, false));
    }

    #[test]
    fn an_odd_number_of_half_cells_ends_on_a_half_cell() {
        assert_eq!(of(44.0, 8), fill(4, true));
        assert_eq!(of(6.0, 8), fill(1, true));
        assert_eq!(of(94.0, 8), fill(8, true));
    }

    #[test]
    fn any_value_above_zero_fills_one_half_cell_at_least() {
        assert_eq!(of(0.5, 8), fill(1, true));
        assert_eq!(of(0.0, 8), fill(0, false));
        assert_eq!(of(-4.0, 8), fill(0, false));
        assert_eq!(of(f64::NAN, 8), fill(0, false));
    }
}
