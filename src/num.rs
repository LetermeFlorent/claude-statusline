// Arrondis sans libm : sous Windows le binaire est en no_std, et core n'a ni
// floor ni round. Au-dela de 2^52 un f64 est deja entier.

const EXACT: f64 = 4503599627370496.0;

pub fn abs(x: f64) -> f64 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

pub fn trunc(x: f64) -> f64 {
    if !(abs(x) < EXACT) {
        return x;
    }
    (x as i64) as f64
}

pub fn floor(x: f64) -> f64 {
    let t = trunc(x);
    if t > x {
        t - 1.0
    } else {
        t
    }
}

pub fn ceil(x: f64) -> f64 {
    let t = trunc(x);
    if t < x {
        t + 1.0
    } else {
        t
    }
}

// f64::round : la moitie part loin de zero
pub fn round(x: f64) -> f64 {
    let t = trunc(x);
    let d = x - t;
    if d >= 0.5 {
        t + 1.0
    } else if d <= -0.5 {
        t - 1.0
    } else {
        t
    }
}

pub fn clamp(x: f64, lo: f64, hi: f64) -> f64 {
    if x < lo {
        lo
    } else if x > hi {
        hi
    } else {
        x
    }
}

pub fn is_integral(x: f64) -> bool {
    trunc(x) == x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrondis() {
        assert_eq!(round(2.5), 3.0);
        assert_eq!(round(-2.5), -3.0);
        assert_eq!(round(0.49999999999999994), 0.0);
        assert_eq!(floor(-0.5), -1.0);
        assert_eq!(ceil(1.0000001), 2.0);
        assert_eq!(ceil(-1.5), -1.0);
        assert_eq!(floor(1e300), 1e300);
    }
}
