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

    /// Returns point of the fibre on the S(3) hyper-sphere.
    #[allow(non_snake_case)]
    #[must_use]
    pub fn build(&self, n: u32) -> impl ExactSizeIterator<Item = (f64, f64, f64, f64)> {
      let delta = (self.alpha_end - self.alpha_start) / (n as f64);
      (0..n).map(move |i| {
        let alpha = f64::from(i) * delta;

            let X0 = ((alpha + self.phi) / 2_f64).cos() * (self.theta / 2_f64).sin();
            let X1 = ((alpha + self.phi) / 2_f64).sin() * (self.theta / 2_f64).sin();
            let X2 = ((alpha - self.phi) / 2_f64).cos() * (self.theta / 2_f64).cos();
            let X3 = ((alpha - self.phi) / 2_f64).sin() * (self.theta / 2_f64).cos();
            (X0, X1, X2, X3)
        })
    }
}
