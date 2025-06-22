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

use hopf::generate_obj;

fn main() {
    // TODO Take seed from stdIn.
    let mut seeds = vec![];

    let lat = 10_f64.to_radians();
    (0..180).step_by(10).for_each(|i| {
        let lon = (0_f64 + 10_f64 + f64::from(i)).to_radians();
        seeds.push((lat, lon));
    });

    let lat = 30_f64.to_radians();
    (0..180).step_by(10).for_each(|i| {
        let lon = (30_f64 + f64::from(i)).to_radians();
        seeds.push((lat, lon));
    });

    let lat = 50_f64.to_radians();
    (0..180).step_by(10).for_each(|i| {
        let lon = (60_f64 + 10_f64 + f64::from(i)).to_radians();
        seeds.push((lat, lon));
    });

    let mut lines = vec![];
    for (lat, lon) in seeds {
        let fibre = hopf::fibre::Fibre::new(lat, lon, 0_f64, 4.0 * std::f64::consts::PI);
        let points = fibre.build(100).map(hopf::project).collect::<Vec<_>>();
        lines.push(points);
    }

    generate_obj(&lines)
        .lines()
        .for_each(|line| println!("{line}"));
}
