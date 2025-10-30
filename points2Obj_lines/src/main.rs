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

use std::io::{BufWriter, Error};

use hopf::generate_obj_lines;

fn main() -> Result<(), std::io::Error> {
    // TODO Take seed from stdIn.
    let mut seeds = vec![];

    let stdout = std::io::stdout();
    let handle = stdout.lock();
    let mut writer = BufWriter::new(handle);

    let lat = 10_f64.to_radians();
    (0..270).step_by(1).for_each(|i| {
        let lon = (0_f64 + 10_f64 + f64::from(i)).to_radians();
        seeds.push((lat, lon));
    });

    let lat = 20_f64.to_radians();
    (0..270).step_by(5).for_each(|i| {
        let lon = (30_f64 + f64::from(i)).to_radians();
        seeds.push((lat, lon));
    });

    let lat = 30_f64.to_radians();
    (0..270).step_by(10).for_each(|i| {
        let lon = (60_f64 + 10_f64 + f64::from(i)).to_radians();
        seeds.push((lat, lon));
    });

    let mut lines = vec![];
    for (lat, lon) in seeds {
        let fibre = hopf::fibre::Fibre::new(lat, lon, 0_f64..4.0 * std::f64::consts::PI);
        // let points = fibre.adaptive_build(1000);
        let (points, _) = fibre.build(1_f64, 40, 2000_u32).map_err(|_| {
            std::io::Error::other("Oscillation detected while adaptively constructing a fibre")
        })?;
        lines.push(points);
    }

    generate_obj_lines(&lines, &mut writer).map_err(|_| Error::other("Error writing output."))
}
