use core::ops::Range;
use core::{error::Error, f32};

use core::ops::RangeInclusive;
use std::fmt::Display;
use std::fmt::Formatter;

use crate::Vertex;
use crate::length::resample_fibre;
use crate::sp::SurfacePoint;
use crate::{length::path_length, project};

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
    sp: SurfacePoint,

    // alpha [0..=4PI] is the domain of the fibre.
    //
    // NB alpha=0 is the same point as alpha=4PI.
    // This duplication is useful when defining a closed path.
    alpha: RangeInclusive<f32>,
}

/// Setting extarcted from polar coords.
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
    /// alpha must be equal to or contained by 0..4PI.
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

        Self { sp, alpha }
    }

    /// Returns a points on the fibre (uniformly separated).
    ///
    /// `target_samples` - The number of on the closed path (max value 65,535).
    ///                  - Limited to u16 for convience in casting.
    ///                  - u32 cannot be easily cast to a f32.
    ///
    /// `n_tries` - is the maximum number of tries in step size adjustment loop.
    ///
    /// The points are uniformly separated by adaptively altering alpha
    /// until adjacent points are separated by a value a error of 1%.
    ///
    /// Hardcoded sensitive parameters :-
    ///
    /// The step size adjustment is up of down by 10%.
    ///
    /// # Errors
    ///
    /// When the size up and step down adjustment oscillates
    /// between two values and `n_tries` is exceeded.
    pub fn build(
        &self,
        target_samples: u16,
        n_tries: u16,
    ) -> Result<(Vec<Vertex>, Vec<f32>), FibreBuildError> {
        const TOLLERANCE: f32 = 0.001_f32;

        let fibre = self.projected_fibre();
        // Target number of points per circle.
        let len = path_length(&fibre, &self.alpha, 10_000);

        // Target distance to travel per step;
        let target_dist = len / f32::from(target_samples);

        let delta = TOLLERANCE * target_dist;

        let distance_min = target_dist - delta;
        let distance_max = target_dist + delta;

        // Change in alpha. Dynamically adjusted step size.
        let mut step = 4_f32 * f32::consts::PI / f32::from(target_samples);

        let mut f_last = fibre(*self.alpha.start());
        let mut alpha_last = *self.alpha.start();

        let mut points = Vec::with_capacity(target_samples as usize);
        let mut alphas = Vec::with_capacity(target_samples as usize);

        let mut i;
        'outer: loop {
            // Adjust step size.
            let mut f;
            let mut alpha;
            i = 0;
            'adaptive_loop: loop {
                // paranoia - clamp
                alpha = alpha_last + step;

                f = fibre(alpha);

                if alpha > *self.alpha.end() {
                    break 'adaptive_loop;
                }

                let d = f_last.0.distance(f.0);
                if d > distance_max {
                    step *= 0.8_f32; // Too fast, reduce step size.
                } else if d < distance_min {
                    step *= 1.2_f32; // Too slow, increase step size.
                } else {
                    // Upon exit, alpha is the last value used.
                    break 'adaptive_loop; // Acceptable velocity, break inner loop.
                }

                if i > n_tries {
                    return Err(FibreBuildError::NTriesExceed(n_tries));
                }
                i += 1;
            }

            f_last = f;
            alpha_last = alpha;
            points.push(f);
            alphas.push(alpha);

            if alpha >= *self.alpha.end() {
                break 'outer;
            }
        }

        // if i != n_tries {
        //     return Err(FibreBuildError::NTriesExceed(i));
        // }

        Ok((points, alphas))
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
    pub fn build_uniform<const M: usize>(&self) -> (Vec<Vertex>, Vec<f32>) {
        let lut = resample_fibre::<1024, M>(self.projected_fibre(), &self.alpha);

        let path_length = lut[M - 1].1;
        let step = path_length / M as f32;

        let fibre = self.projected_fibre();

        let mut last_match = 0_usize;
        (0..M)
            .map(|i| {
                // a
                let dist_threshold = i as f32 * step;

                // Preformance: Search through a progressively smaller section of the LUT.
                let (match_index, (alpha, _dist)) = lut[last_match..]
                    .iter()
                    .enumerate()
                    .find(move |(_i, (_alpha, d))| {
                        // Threshold distance.
                        *d >= dist_threshold
                    })
                    .unwrap_or((0_usize, &(f32::NAN, f32::NAN)));

                last_match = match_index;
                (fibre(*alpha), alpha)
            })
            .unzip()
    }

    // Perform binary search on the curve to get uniform evenly spaced point in the output domain
    //
    // Requires further work, before use.
    // pub fn build_binary(
    //     &self,
    //     scale: f64,
    //     target_samples: u32,
    //     n_tries: u32,
    // ) -> (Vec<Vertex>, Vec<f64>) {
    //     const TOLLERANCE: f64 = 0.001_f64;

    //     let fibre = self.projected_fibre();
    //     // Target number of points per circle.
    //     let len = path_length(&fibre, &self.alpha, 10_000);

    //     // Target distance to travel per step;
    //     let target_dist = len / f64::from(target_samples);

    //     let delta = TOLLERANCE * target_dist;

    //     let distance_min = target_dist - delta;
    //     let distance_max = target_dist + delta;
    //     // Using a Range to form a bracket
    //     let target_dist_range = RangeInclusive::new(distance_min, distance_max);

    //     // Change in alpha. Dynamically adjusted step size.
    //     let mut step = 4_f64 * f64::consts::PI / f64::from(target_samples);

    //     let mut f_last = fibre(*self.alpha.start());
    //     let mut alpha_last = *self.alpha.start();

    //     let mut points = Vec::with_capacity(target_samples as usize);
    //     let mut alphas = Vec::with_capacity(target_samples as usize);
    //     let alpha_start = *self.alpha.start();

    //     // Set the bracket to be (alpha_start, alpha_mid)
    //     //
    //     // Assumptions for initial bracket.
    //     // the first point is contained beteen the start and the midpoint alpha vlaue.
    //     // Cannot just used the end point as for closed paths the start and the end are identical.
    //     let alpha_lower_bound = alpha_last;
    //     let alpha_upper_bound = alpha_last.midpoint(*self.alpha.end());

    //     // First point requires no computation.
    //     points.push(fibre(alpha_start));
    //     alphas.push(alpha_start);
    //     for i in 1..target_samples {
    //         // let a = alpha_start + i as f64 * step;
    //         let a = (i as f64).mul_add(step, alpha_start);
    //         let f_lower = fibre(alpha_lower_bound);
    //         let f_upper = fibre(alpha_upper_bound);

    //         // binary search until the point is within tollerance.
    //         'search_loop: for i in 0..n_tries {
    //             // euclidean distance between points, is a cheap subsitute
    //             // for the distance along the path.
    //             let dist = (f_lower - f_last).length();

    //             if dist > distance_max {
    //                 // bring the upper value down
    //             } else if dist < distance_min {
    //                 // raise lower bound.
    //             } else {
    //                 // the distance is within tollerances.
    //                 break 'search_loop;
    //             }
    //         }
    //         // The upper and lower bound are effective the same point
    //         // use the lower bound a the result.
    //         //
    //         let f_last = f_lower;
    //         // Scale the output, not the search data.
    //         points.push(f_last * scale);
    //         alphas.push(alpha_lower_bound);
    //     }

    //     (points, alphas)
    // }

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

    use super::*;

    #[test]
    fn fibre_build() {
        let fibre = Fibre::new(
            SurfacePoint {
                lat: 5.0_f32.to_radians(),
                lon: 5.0_f32.to_radians(),
            },
            0_f32..=4.0 * core::f32::consts::PI,
        );

        match fibre.build(1_000, 2000) {
            Ok((points, _)) => {
                assert_eq!(points.len(), 1_000);
            }
            Err(_) => {
                assert!(false);
            }
        }
    }

    #[test]
    fn fibre_build_tight_tolerance() {
        // Numerically unstable test case.
        // Fibre::build()
        //
        // For a given tolerance - Shorter fibre lengths require a tighter delta.
        let fibre = Fibre::new(
            SurfacePoint {
                lat: 5.0_f32.to_radians(),
                lon: 5.0_f32.to_radians(),
            },
            0_f32..=2_f32 * core::f32::consts::PI,
        );

        match fibre.build(1_000, 2_000) {
            Ok((points, _)) => {
                assert_eq!(points.len(), 1_000);
            }
            Err(_) => {
                assert!(false);
            }
        }
    }
}
