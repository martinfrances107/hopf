use core::ops::Range;
use core::{error::Error, f32};

use core::ops::RangeInclusive;
use std::fmt::Display;
use std::fmt::Formatter;

use crate::Vertex;
use crate::length::resample_fibre;
use crate::project;
use crate::sp::SurfacePoint;

// The domain of a fibre is 0..4PI
static ALPHA_MAX: f32 = 4_f32 * core::f32::consts::PI;

// The Range is not 'inclusive'  as it blocks the north pole.
// Under sterographic the projection of a point at the north pole is undefined.
static LAT_RANGE: Range<f32> = -core::f32::consts::FRAC_PI_2..core::f32::consts::FRAC_PI_2;
// 0 degrees and 360 degrees which are identical longitudes.
// This range is inclusive to allow for closed paths.
static LON_RANGE: RangeInclusive<f32> = 0_f32..=core::f32::consts::TAU;

/// A fibre is a point on s(2)
/// where alpha extends the fibre from the base space.
#[derive(Debug)]
pub struct Fibre {
    // alpha [0..=4PI] is the domain of the fibre.
    //
    // NB alpha=0 is the same point as alpha=4PI.
    // This duplication is useful when defining a closed path.
    alpha: RangeInclusive<f32>,

    sp: SurfacePoint,
}

/// Setting extracted from polar coords.
#[derive(Debug)]
struct Settings {
    η: f32,
    ξ1: f32,
}

/// Adaptive step size failure
///
/// The step size adjustment is oscillating between two values.
#[derive(Debug, Clone)]
// pub struct NTriesExceedError;
pub enum FibreBuildError {
    /// Too many tries adjusting step size.
    NTriesExceed(u16),
    /// Too few tries allowed adjusting step size.
    NTriesTooLow(u16),
}

impl Error for FibreBuildError {}

impl Display for FibreBuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SuperError is here!")
    }
}

impl Fibre {
    /// Create a new fibre.
    ///
    /// Alpha must be equal to or contained by 0..=4PI.
    ///
    /// NB. This will only be checked in debug builds.
    #[must_use = "Not using the returned, is the same as doing nothing at all."]
    pub fn new(sp: SurfacePoint, alpha: RangeInclusive<f32>) -> Self {
        debug_assert!(LAT_RANGE.contains(&sp.lat), "lat {:#?}", sp.lat);
        debug_assert!(LON_RANGE.contains(&sp.lon), "lon {:#?}", sp.lon);

        debug_assert!(*alpha.start() >= 0_f32, "alpha start {:#?}", alpha.start());
        debug_assert!(
            *alpha.start() <= ALPHA_MAX,
            "alpha_start {:#?}",
            alpha.start()
        );

        debug_assert!(*alpha.end() >= 0_f32, "alpha_end {:#?}", alpha.end());
        debug_assert!(*alpha.end() <= ALPHA_MAX, "alpha_end {:#?}", alpha.end());

        Self { alpha, sp }
    }

    /// RAW Uniformly space in domain space results in highly un-evenly spaced output.
    /// NB: Fast but results are almost never what is wanted.
    #[must_use]
    pub fn build_raw(
        &self,
        target_samples: u16,
        n_tries: u16,
    ) -> impl ExactSizeIterator<Item = Vertex> {
        let fibre = self.projected_fibre();

        let step = 4_f32 * f32::consts::PI / f32::from(target_samples);
        let alpha_start = *self.alpha.start();

        (0..n_tries).map(move |i| {
            // let a = alpha_start + i as f64 * step;
            let a = f32::from(i).mul_add(step, alpha_start);
            fibre(a)
        })
    }

    /// Returns points unformly space along the curve.
    ///
    /// This version is memory hungry
    /// TODO Could move the LUT into Self to avoid constant memory allocation/deallocation.
    /// TODO Could remove scaling. blender allows for a scaling at the obj level.
    #[must_use]
    pub fn build_uniform<const N_POINTS_PER_LOOP: usize>(&self) -> (Vec<Vertex>, Vec<f32>) {
        let lut = resample_fibre::<4096, N_POINTS_PER_LOOP>(self.projected_fibre(), &self.alpha);

        let path_length = lut[N_POINTS_PER_LOOP - 1].1;
        let step = path_length / (N_POINTS_PER_LOOP - 1) as f32;

        let fibre = self.projected_fibre();

        let mut last_match = 0_usize;

        (0..N_POINTS_PER_LOOP)
            .map(|i| {
                // a
                let dist_threshold = i as f32 * step;

                // Preformance: use last_match to search through a progressively smaller section of the LUT.
                let (match_index, (alpha, _dist)) =
                    match lut[last_match..]
                        .iter()
                        .enumerate()
                        .find(move |(_i, (_alpha, d))| {
                            // Threshold distance.
                            *d >= dist_threshold
                        }) {
                        Some((index, (a, d))) => (index, (*a, *d)),
                        None => {
                            // Not found! ... Re-examine endpoint with a different test
                            // if the dist is slightly under threshold still match.
                            if let Some((last_alpha, last_dist)) = lut.last() {
                                if (last_dist - dist_threshold).abs() < 1e-2 {
                                    (lut.len(), (*last_alpha, *last_dist))
                                } else {
                                    (0usize, (f32::NAN, f32::NAN))
                                }
                            } else {
                                (0usize, (f32::NAN, f32::NAN))
                            }
                        }
                    };

                last_match = match_index;
                (fibre(alpha), alpha)
            })
            .unzip()
    }

    // Solve for ξ1 and η.
    // Given a point on s2 (lat, long)
    //
    // z = cos(2η)
    // x = sin(2η)cos(ξ1)
    // y = sin(2η)sin(ξ1)
    fn extract_settings(&self) -> Settings {
        let (sin_lat, cos_lat) = self.sp.lat.sin_cos();
        let cos_lon = self.sp.lon.cos();

        // polar coords to cartesian.
        let x = cos_lat * cos_lon;
        // let _y = cos_lat * sin_lon;
        let z = sin_lat;

        let η = z.acos() / 2.0;
        let sin2n = (2.0 * η).sin();
        let x_div_sin2n = x / sin2n;
        let ξ1 = (x_div_sin2n).acos();

        Settings { η, ξ1 }
    }

    /// Transform a "time", t parameter into a point in E^3
    ///
    /// <https://en.wikipedia.org/wiki/Hopf_fibration>
    ///
    /// 0<= η <= pi/2
    /// 0<= ξ1 <= 2 * pi
    /// 0<= ξ2 <= 4 * pi
    ///
    /// The "use<> implies "capture nothing"
    /// <https://rust-lang.github.io/rfcs/3617-precise-capturing.html>
    #[allow(non_snake_case)]
    pub fn projected_fibre(&self) -> impl use<> + Fn(f32) -> Vertex {
        let Settings { η, ξ1 } = self.extract_settings();

        let (sin_η, cos_η) = η.sin_cos();
        // The domain of ξ2 is 0..4PI
        move |ξ2| {
            let X1 = f32::midpoint(ξ1, ξ2).cos() * sin_η;
            let X2 = f32::midpoint(ξ1, ξ2).sin() * sin_η;
            let X3 = ((ξ2 - ξ1) / 2_f32).cos() * cos_η;
            let X4 = ((ξ2 - ξ1) / 2_f32).sin() * cos_η;
            project(X1, X2, X3, X4)
        }
    }
}

#[cfg(test)]
mod tests {

    use approx::relative_eq;

    use crate::F32_4PI;

    use super::*;

    #[test]
    /// Due to the cycling nature of the fibre 0 and 4*PI are the same point.
    fn projected() {
        let fibre = Fibre::new(
            SurfacePoint {
                lat: 5.0_f32.to_radians(),
                lon: 5.0_f32.to_radians(),
            },
            0_f32..=F32_4PI,
        );

        let fibre = fibre.projected_fibre();
        let at_zero = fibre(0_f32);
        let at_4pi = fibre(F32_4PI);
        let delta = (at_zero - at_4pi).length();
        assert!(
            delta < 1e-6,
            "Failed 0 and 4PI are the same point {at_zero:#?} {at_4pi:#?} delta {delta}"
        );
    }

    /// Loop up table test.
    ///
    /// If a fibre runs from 0..=4PI then the first and last points
    /// must be indetical ( or with a small error)
    #[test]
    fn lut() {
        let fibre = Fibre::new(
            SurfacePoint {
                lat: 5.0_f32.to_radians(),
                lon: 5.0_f32.to_radians(),
            },
            0_f32..=F32_4PI,
        );

        let (points, alphas) = fibre.build_uniform::<1000>();

        // Check alphas
        //
        // Start to zero.
        let expecting_zero = (alphas[0] - 0_f32).abs();
        assert!(expecting_zero < 1e-6);

        // End at 4PI.
        let alpha_last = alphas.last().unwrap();
        // 1 part in 100 ... this seems loose.
        assert!(
            relative_eq!(*alpha_last, F32_4PI, max_relative = 1e-2),
            "observed {alpha_last}, expected {F32_4PI} "
        );

        // Check points vector.
        let first_point = points.first().unwrap();
        let last_point = points.last().unwrap();

        let delta = (*first_point - *last_point).length();
        assert!(
            delta < 1e-1,
            "for a close path the first and last points must be close {first_point:#?} {last_point:#?} {delta}"
        );
    }
}
