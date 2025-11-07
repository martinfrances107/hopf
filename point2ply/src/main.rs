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

use hopf::{fibre::Fibre, generate_ply, sp::SurfacePoint};
use std::io::{BufWriter, Error};

fn main() -> Result<(), Error> {
    // TODO Take seed from stdIn.

    let stdout = std::io::stdout();
    let handle = stdout.lock();
    let mut writer = BufWriter::new(handle);

    let fibre = Fibre::new(
        SurfacePoint {
            lat: 5.0_f32.to_radians(),
            lon: 5.0_f32.to_radians(),
        },
        0_f32..=4.0 * core::f32::consts::PI,
    );

    let (points, _) = fibre
        .build(20, u16::MAX)
        .map_err(|_| Error::other("Oscillation detected while adaptively constructing a fibre"))?;

    // let points = fibre.build_raw(1_f64, 20, 2_000_u32);

    // let (points, _) = fibre.build_uniform::<10>(1_f64);

    generate_ply(points.into_iter(), &mut writer)
        .map_err(|_| Error::other("Fail to write to buffer"))
}
