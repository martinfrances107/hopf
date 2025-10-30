use crate::binary_search::Bracket;
use crate::binary_search::binary_search;
use nrfind::find_root;

// Numerical differentiation using the central difference method
// This approximates the gradient (derivative) for F.
fn central_difference<F>(f: F, x: f64, h: f64) -> f64
where
    F: Fn(f64) -> f64,
{
    (f(x + h) - f(x - h)) / (2.0 * h)
}

/// Reparametrize function f to have uniform steps in output space.
fn reparametrize(f: &dyn Fn(f64) -> f64, x_start: f64, x_end: f64, n_out: usize) -> Vec<f64> {
    let dist = 0_f64;

    const N_DETAILED: usize = 1000usize;
    const N_DETAILED_FLOAT: f64 = N_DETAILED as f64;
    let x_arr = (0..=N_DETAILED)
        .map(|i| x_start + (x_end - x_start) * (i as f64) / N_DETAILED_FLOAT)
        .collect::<Vec<_>>();

    // println!("x_arr {:#?}", x_arr);

    let mut dist = 0_f64;
    let mut dist_arr = Vec::with_capacity(N_DETAILED);
    dist_arr.push(0_f64);

    for i in 1..=N_DETAILED {
        let dx = x_arr[i] - x_arr[i - 1];
        let df = f(x_arr[i]) - f(x_arr[i - 1]);
        let ds = (dx * dx + df * df).sqrt();
        dist += ds;
        dist_arr.push(dist);
    }
    println!("arc length {}", dist_arr.last().unwrap());
    let delta = (f(x_end) - f(x_start)) / (n_out as f64);

    (0..=n_out)
        .map(|i| {
            let target = delta * i as f64;
            print!("i {i}, target {:#}", target);
            match binary_search(&dist_arr, &target) {
                Bracket::Exact(i) => {
                    println!("({i})");
                    x_arr[i]
                }
                Bracket::Between(low, high) => {
                    println!(
                        "({low}, {high}) - ( dist {} {}, x {} {})",
                        dist_arr[low], dist_arr[high], x_arr[low], x_arr[high]
                    );
                    // Express the target distance as value between [0..1] within the bracket.
                    let lerp_inv = (target - dist_arr[low]) / (dist_arr[high] - dist_arr[low]);
                    // Interpolate x accordingly.
                    let out = x_arr[low] + lerp_inv * (x_arr[high] - x_arr[low]);
                    println!(
                        "({low}, {high}) - ( dist {} {}, x {} {}) out {}",
                        dist_arr[low], dist_arr[high], x_arr[low], x_arr[high], out
                    );
                    out
                }
                Bracket::OutOfBounds => {
                    panic!("Value out of bounds during normalization {}", target)
                }
            }
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parabolic() {
        let f = |x: f64| x * x * x;

        let uniform = reparametrize(&f, 0_f64, 10_f64, 10);

        println!("uniform {:#?}", uniform);
        assert!(false);
        let expected_separation = 10_f64;
        for (j, u) in uniform.windows(2).enumerate() {
            let sep = u[1] - u[0];
            let diff = (sep - expected_separation).abs();
            assert!(
                diff < 1e-6,
                "Separation at index {j} differs by {diff}, got {sep}, expected {expected_separation}"
            );
        }
    }
}
