//! A library for generating a Hopf fibration surface from a complex number
#![deny(clippy::all)]
#![warn(clippy::cargo)]
#![warn(clippy::complexity)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::perf)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![allow(clippy::many_single_char_names)]

use num_complex::Complex64;

use hopf::generate_ply;

fn main() {
    // TODO Take seed from stdIn.
    let seed = Complex64::from_polar(1_f64, 45_f64.to_radians());

    let fibre = hopf::fibre::Fibre::new(seed.re, seed.im, 0_f64, 4.0 * std::f64::consts::PI);

    let gen_points = fibre.build(100).map(hopf::project);

    generate_ply(gen_points)
        .lines()
        .for_each(|line| println!("{line}"));
}
