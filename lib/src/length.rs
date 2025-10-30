use core::ops::Range;

use crate::Vertex;

/// Crude estimate of length bases on n step.
///
/// alpha is not "constant velocity" so the step size
/// is highly variable.
///
/// As n -> infinity, the output -> length
pub(crate) fn path_length(f: impl Fn(f64) -> Vertex, alpha_range: &Range<f64>, n: u32) -> f64 {
    let mut alpha = alpha_range.start;
    let mut f_last = f(alpha_range.start);
    let step = (alpha_range.end - alpha_range.start) / f64::from(n);

    (1..n).fold(0_f64, |acc, _| {
        alpha += step;
        let f = f(alpha);
        let d = f_last.0.distance(f.0);
        f_last = f;
        acc + d
    })
}

#[cfg(test)]
mod tests {
    use core::f64;

    use glam::DVec3;

    use crate::length;

    use super::*;

    fn circle(alpha: f64) -> Vertex {
        Vertex(DVec3 {
            x: alpha.sin(),
            y: alpha.cos(),
            z: 0_f64,
        })
    }

    #[test]
    fn length_arcs() {
        let len = path_length(circle, &(0_f64..2_f64 * f64::consts::PI), 1_000_000);
        let expected = 2_f64 * f64::consts::PI;
        assert!((len - expected).abs() < 1e-5);

        let len = path_length(circle, &(0_f64..f64::consts::PI / 2_f64), 1_000_000);
        let expected = f64::consts::PI / 2_f64;
        assert!((len - expected).abs() < 1e-5);
    }
}
