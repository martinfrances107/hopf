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
    f: impl Fn(f32) -> Vertex,
    alpha_range: &RangeInclusive<f32>,
    n: u16,
) -> f32 {
    let mut alpha = *alpha_range.start();
    let mut f_last = f(alpha);
    let step = (alpha_range.end() - alpha_range.start()) / f32::from(n);

    (1..n).fold(0_f32, |acc, _| {
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
pub(crate) fn searchable_path_length<const N_POINTS_PER_LOOP: usize>(
    fibre: impl Fn(f32) -> Vertex,
    alpha_range: &RangeInclusive<f32>,
) -> [(f32, f32); N_POINTS_PER_LOOP] {
    // Crazy casting rules is there a better way
    let n_16 = u16::try_from(N_POINTS_PER_LOOP).expect("N MUST be less than 65,535");
    let n_f32 = f32::from(n_16);

    let alpha_step = (alpha_range.end() - alpha_range.start()) / n_f32;

    let alpha_start = *alpha_range.start();
    let mut f_last = fibre(alpha_start);
    let mut d = 0_f32;
    array::from_fn(move |i_usize| {
        let i = u16::try_from(i_usize).expect("N MUST be limited to 65,535");
        let alpha = f32::from(i).mul_add(alpha_step, alpha_start);
        let f = fibre(alpha);
        d += (f - f_last).length();
        f_last = f;
        (alpha, d)
    })
}

// Returns a coarse set of (alpha, distance) values
// computed from fine grained sampling.
pub(crate) fn resample_fibre<const N_DETAILED: usize, const N_COARSE: usize>(
    fibre: impl Fn(f32) -> Vertex,
    alpha_range: &RangeInclusive<f32>,
) -> [(f32, f32); N_COARSE] {
    debug_assert!(N_DETAILED > N_COARSE);
    // (alpha, path length) look up table - LUT.
    //
    // Fine sample of fibre.
    let lut = searchable_path_length::<N_DETAILED>(fibre, alpha_range);
    let m_32 = N_COARSE as u32;
    let step = lut[N_DETAILED - 1].1 / (m_32 - 1) as f32;
    // Reduce to a unformly separated set.
    array::from_fn(move |i| {
        let dist_threshold = i as f32 * step;
        let (alpha, dist) = match lut.iter().find(|&&(_, d)| {
            // Threshold distance.
            d >= dist_threshold
        }) {
            Some((a, d)) => (*a, *d),
            None => {
                // Not found! ... Re-examine endpoint with a different test
                // if the dist is slightly under threshold still match.
                if let Some((last_alpha, last_dist)) = lut.last() {
                    if (last_dist - dist_threshold).abs() < 1e-3 {
                        (*last_alpha, *last_dist)
                    } else {
                        (f32::NAN, f32::NAN)
                    }
                } else {
                    (f32::NAN, f32::NAN)
                }
            }
        };

        (alpha, dist)
    })
}

#[cfg(test)]
mod tests {
    use core::f32;

    use glam::Vec3;

    use super::*;

    fn circle(alpha: f32) -> Vertex {
        Vertex(Vec3 {
            x: alpha.cos(),
            y: alpha.sin(),
            z: 0_f32,
        })
    }

    #[test]
    fn half_circle() {
        let len = path_length(circle, &(0_f32..=f32::consts::PI / 2_f32), u16::MAX);
        let expected = f32::consts::PI / 2_f32;
        let rel_diff = (len - expected).abs() / expected;
        assert!(
            rel_diff < 1e-3,
            "len {len} expected {expected} frational difference {rel_diff} "
        );
    }

    #[test]
    fn full_circle() {
        let len = path_length(circle, &(0_f32..=2_f32 * f32::consts::PI), u16::MAX);
        let expected = 2_f32 * f32::consts::PI;
        let rel_diff = (len - expected).abs() / expected;
        assert!(
            rel_diff < 1e-3,
            "len {len} expected {expected} frational difference {rel_diff} "
        );
    }

    // Use a unit circle to confirm points are searchable.
    #[test]
    fn searchable() {
        static N: usize = 24 * 1024;
        let path_store = searchable_path_length::<N>(circle, &(0_f32..=f32::consts::TAU));

        // Search for a point quater of the way around the circle.
        let &(alpha, _) = path_store
            .iter()
            .find(|&&(_, dist)| {
                // Threshold distance.
                dist >= core::f32::consts::FRAC_PI_2
            })
            .unwrap_or(&(f32::NAN, f32::NAN));
        let rel_error = (core::f32::consts::FRAC_PI_2 - alpha).abs() / core::f32::consts::FRAC_PI_2;
        println!("error {}", rel_error);
        assert!(rel_error < 1e-3);

        // Search for a point half way around the circle.
        let &(alpha, _) = path_store
            .iter()
            .find(|&&(_, dist)| {
                // Threshold distance.
                dist >= core::f32::consts::PI
            })
            .unwrap_or(&(f32::NAN, f32::NAN));
        let rel_error = (core::f32::consts::PI - alpha).abs() / core::f32::consts::PI;
        assert!(rel_error < 1e-4, "error {}", rel_error);

        // Final valus is a expected
        let &(_, max) = path_store.last().unwrap_or(&(f32::NAN, f32::NAN));
        let rel_error = (core::f32::consts::TAU - max).abs() / core::f32::consts::TAU;
        assert!(rel_error < 1e-4, "error {}", rel_error);
    }
}
