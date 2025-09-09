use crate::{Vertex, fibre::distance};

/// Crude estimate of length bases on n step.
///
/// alpha is not "constant velocity" so the step size
/// is highly variable.
///
/// As n -> infinity, the output -> length
pub(crate) fn path_length(
    f: impl Fn(f64) -> Vertex,
    alpha_start: f64,
    alpha_end: f64,
    n: u32,
) -> f64 {
    let mut alpha = alpha_start;
    let mut f_last = f(alpha_start);
    let step = (alpha_end - alpha_start) / f64::from(n);

    let mut len = 0_f64;
    for _ in 1..n {
        alpha += step;
        let f = f(alpha);
        let d = distance(&f_last, &f);
        len += d;
        f_last = f;
    }
    len
}

#[cfg(test)]
mod tests {
    use core::f64;

    use crate::length;

    use super::*;

    fn circle(alpha: f64) -> (f64, f64, f64) {
        (alpha.sin(), alpha.cos(), 0_f64)
    }

    #[test]
    fn length_arcs() {
        let len = path_length(circle, 0_f64, 2_f64 * f64::consts::PI, 1_000_000);
        let expected = 2_f64 * f64::consts::PI;
        assert!((len - expected).abs() < 1e-5);

        let len = path_length(circle, 0_f64, f64::consts::PI / 2_f64, 1_000_000);
        let expected = f64::consts::PI / 2_f64;
        assert!((len - expected).abs() < 1e-5);
    }
}
