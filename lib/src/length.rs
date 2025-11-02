use core::ops::RangeInclusive;
use std::array;

use crate::Vertex;

/// Crude estimate of length bases on n step.
///
/// alpha is not "constant velocity" so the step size
/// is highly variable.
///
/// As n -> infinity, the output -> length
pub(crate) fn path_length(
    f: impl Fn(f64) -> Vertex,
    alpha_range: &RangeInclusive<f64>,
    n: u32,
) -> f64 {
    let mut alpha = *alpha_range.start();
    let mut f_last = f(alpha);
    let step = (alpha_range.end() - alpha_range.start()) / f64::from(n);

    (1..n).fold(0_f64, |acc, _| {
        alpha += step;
        let f = f(alpha);
        let d = f_last.0.distance(f.0);
        f_last = f;
        acc + d
    })
}
// Returns a mapping for reparameterizing a non-uniform curve.
//
// A searchable mapping relating alpha to path length
// return [(alpha, dist); N]
//
// A value of 1024 should be enough to resample
// to 16 evenly spaced points.
pub(crate) fn searchable_path_length<const N: usize>(
    fibre: impl Fn(f64) -> Vertex,
    alpha_range: &RangeInclusive<f64>,
) -> [(f64, f64); N] {
    let n_32 = N as u32;
    let step = (alpha_range.end() - alpha_range.start()) / f64::from(n_32);

    let alpha_start = *alpha_range.start();
    let mut f_last = fibre(alpha_start);
    let mut d = 0_f64;
    array::from_fn(move |i| {
        // let alpha = alpha_start + i as f64 * step;
        let alpha = (i as f64).mul_add(step, alpha_start);
        let f = fibre(alpha);
        d += (f - f_last).length();
        f_last = f;
        (alpha, d)
    })
}

// Returns a coarse set of (alpha, distance) values
// computed from fine grained sampling.
pub(crate) fn resample_fibre<const N: usize, const M: usize>(
    fibre: impl Fn(f64) -> Vertex,
    alpha_range: &RangeInclusive<f64>,
) -> [(f64, f64); M] {
    debug_assert!(N > M);

    // (alpha, path length) look up table.
    //
    // Fine sample of fibre.
    let lut = searchable_path_length::<N>(fibre, alpha_range);

    let m_32 = M as u32;
    let step = lut[N - 1].1 / f64::from(m_32);
    // Reduce to a unformly separated set.
    array::from_fn(move |i| {
        let dist_threshold = i as f64 * step;
        let &(alpha, dist) = lut
            .iter()
            .find(|&&(_, d)| {
                // Threshold distance.
                d >= dist_threshold
            })
            .unwrap_or(&(f64::NAN, f64::NAN));
        (alpha, dist)
    })
}

#[cfg(test)]
mod tests {
    use core::f64;

    use glam::DVec3;

    use super::*;

    fn circle(alpha: f64) -> Vertex {
        Vertex(DVec3 {
            x: alpha.cos(),
            y: alpha.sin(),
            z: 0_f64,
        })
    }

    #[test]
    fn length_arcs() {
        let len = path_length(circle, &(0_f64..=2_f64 * f64::consts::PI), 1_000_000);
        let expected = 2_f64 * f64::consts::PI;
        assert!((len - expected).abs() < 1e-5);

        let len = path_length(circle, &(0_f64..=f64::consts::PI / 2_f64), 1_000_000);
        let expected = f64::consts::PI / 2_f64;
        assert!((len - expected).abs() < 1e-5);
    }

    // For a unit circle alpha is the path length.
    #[test]
    fn searchable() {
        static N: usize = 24 * 1024;
        let path_store = searchable_path_length::<N>(circle, &(0_f64..=2_f64 * f64::consts::PI));

        // Quarter
        let &(alpha, _) = path_store
            .iter()
            .find(|&&(_, dist)| {
                // Threshold distance.
                dist >= core::f64::consts::FRAC_PI_2
            })
            .unwrap_or(&(f64::NAN, f64::NAN));
        let rel_error = (core::f64::consts::FRAC_PI_2 - alpha).abs() / core::f64::consts::FRAC_PI_2;
        println!("error {}", rel_error);
        assert!(rel_error < 1e-3);

        // Half circle.
        let &(alpha, _) = path_store
            .iter()
            .find(|&&(_, dist)| {
                // Threshold distance.
                dist >= core::f64::consts::PI
            })
            .unwrap_or(&(f64::NAN, f64::NAN));
        let rel_error = (core::f64::consts::PI - alpha).abs() / core::f64::consts::PI;
        println!("error {}", rel_error);
        assert!(rel_error < 1e-4);

        // Final valus is a expected
        let &(_, max) = path_store.last().unwrap_or(&(f64::NAN, f64::NAN));
        let rel_error = (core::f64::consts::TAU - max).abs() / core::f64::consts::TAU;
        println!("error {}", rel_error);
        assert!(rel_error < 1e-3);

        // This fails an show that inversion can only be used for something
        // less than but not equal to the final value.
        //
        // Full circle
        // let &(alpha, _) = path_store
        //     .iter()
        //     .find(|&&(_, dist)| {
        //         // Threshold distance.
        //         dist >= core::f64::consts::TAU
        //     })
        //     .unwrap_or(&(f64::NAN, f64::NAN));
        // let rel_error = (core::f64::consts::TAU - alpha).abs() / core::f64::consts::TAU;
        // println!("error {}", rel_error);
        // assert!(rel_error < 1e-4);
    }
}
