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

use hopf::{F32_4PI, generate_obj_lines, sp::SurfacePoint};

fn main() -> Result<(), std::io::Error> {
    // TODO Take seed from stdIn.
    let mut seeds = vec![];

    let stdout = std::io::stdout();
    let handle = stdout.lock();
    let mut writer = BufWriter::new(handle);

    // Big outer shell
    let lat = 10_f32.to_radians();
    (0..270).step_by(1).for_each(|i| {
        let lon = (0_f32 + 10_f32 + i as f32).to_radians();
        seeds.push(SurfacePoint { lat, lon });
    });

    let lat = 20_f32.to_radians();
    (0..270).step_by(5).for_each(|i| {
        let lon = (30_f32 + i as f32).to_radians();
        seeds.push(SurfacePoint { lat, lon });
    });

    let lat = 30_f32.to_radians();
    (0..270).step_by(10).for_each(|i| {
        let lon = (60_f32 + 10_f32 + i as f32).to_radians();
        seeds.push(SurfacePoint { lat, lon });
    });

    let mut lines = vec![];
    let alpha = 0_f32..=F32_4PI;
    for sp in seeds {
        let fibre = hopf::fibre::Fibre::new(sp, &alpha);

        let (points, _) = fibre.build_uniform::<10>();

        lines.push(points);
    }

    generate_obj_lines(&lines, &mut writer).map_err(|_| Error::other("Error writing output."))
}
