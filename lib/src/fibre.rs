use core::{error::Error, f64};
use std::fmt::Display;
use std::fmt::Formatter;

use crate::{length::path_length, project};

/// A fibre is a point on s(2)
/// where alpha extends the fibre from the base space.
#[derive(Debug)]
pub struct Fibre {
    theta: f64,
    phi: f64,
    // alpha is how fibre extends the base space.
    // [0..4PI] is the full fibre.
    alpha_start: f64,
    alpha_end: f64,
}

/// Adaptive step size failure
///
/// The step size adjustment is oscillating between two values.
#[derive(Debug, Clone)]
pub struct NTriesExceedError;

impl Error for NTriesExceedError {}

impl Display for NTriesExceedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SuperError is here!")
    }
}
impl Fibre {
    /// Create a new fibre.
    #[must_use]
    pub fn new(theta: f64, phi: f64, mut alpha_start: f64, mut alpha_end: f64) -> Self {
        alpha_start = alpha_start.clamp(0.0, 4.0 * std::f64::consts::PI);
        alpha_end = alpha_end.clamp(alpha_start, 4.0 * std::f64::consts::PI);

        Self {
            theta,
            phi,
            alpha_start,
            alpha_end,
        }
    }

    /// Returns a points on the fibre (uniformly separated).
    ///
    /// `target_samples` - The number of on the closed path
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
        target_samples: u32,
        n_tries: u32,
    ) -> Result<Vec<(f64, f64, f64)>, NTriesExceedError> {
        let fibre = self.projected_fibre();
        // Target number of points per circle.
        let len = path_length(&fibre, 0_f64, 4_f64 * f64::consts::PI, 10_000);

        // Target distance to travel per step;
        let target_dist = len / f64::from(target_samples);
        let distance_min = 0.99 * target_dist;
        let distance_max = 1.01 * target_dist;

        // Change in alpha. Dynamically adjusted step size.
        let mut step = 4_f64 * f64::consts::PI / f64::from(target_samples);

        let mut f_last = fibre(self.alpha_start);
        let mut alpha_last = self.alpha_start;

        let mut points = vec![];
        'outer: loop {
            // Adjust step size.
            let mut f;
            let mut alpha;
            let mut i = 0;
            'adaptive_loop: loop {
                // paranoia - clamp
                alpha = (alpha_last + step).clamp(self.alpha_start, self.alpha_end);

                f = fibre(alpha);

                if alpha >= self.alpha_end {
                    break 'adaptive_loop;
                }
                let d = distance(f_last, f);
                if d > distance_max {
                    step *= 0.8_f64; // Too fast, reduce step size.
                } else if d < distance_min {
                    step *= 1.2_f64; // Too slow, increase step size.
                } else {
                    // Upon exit, alpha is the last value used.
                    break 'adaptive_loop; // Acceptable velocity, break inner loop.
                }

                if i > n_tries {
                    return Err(NTriesExceedError);
                }
                i += 1;
            }

            f_last = f;
            alpha_last = alpha;
            points.push(f);
            if alpha >= self.alpha_end {
                break 'outer;
            }
        }
        Ok(points)
    }

    // Transform a "time", t parameter into a point in E^3
    #[allow(non_snake_case)]
    fn projected_fibre(&self) -> impl Fn(f64) -> (f64, f64, f64) {
        move |t| {
            let X0 = f64::midpoint(t, self.phi).cos() * (self.theta / 2_f64).sin();
            let X1 = f64::midpoint(t, self.phi).sin() * (self.theta / 2_f64).sin();
            let X2 = ((t - self.phi) / 2_f64).cos() * (self.theta / 2_f64).cos();
            let X3 = ((t - self.phi) / 2_f64).sin() * (self.theta / 2_f64).cos();
            project(X0, X1, X2, X3)
        }
    }
}

/// Euclidean distance between two points in 3d space.
pub(crate) fn distance(f0: (f64, f64, f64), f1: (f64, f64, f64)) -> f64 {
    let dx = f1.0 - f0.0;
    let dy = f1.1 - f0.1;
    let dz = f1.2 - f0.2;
    dz.mul_add(dz, dx.mul_add(dx, dy * dy)).sqrt()
}

#[cfg(test)]
mod tests {
    use core::fmt::Error;

    use super::*;

    #[test]
    fn fibre_build() {
        let fibre = Fibre::new(
            5.0_f64.to_radians(),
            5.0_f64.to_radians(),
            0_f64,
            4.0 * std::f64::consts::PI,
        );

        match fibre.build(12, 1_000_u32) {
            Ok(points) => {
                assert_eq!(points.len(), 12);
            }
            Err(_) => {
                assert!(false);
            }
        }
    }
}
