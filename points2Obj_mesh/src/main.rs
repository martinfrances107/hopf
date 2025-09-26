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

use hopf::fibre::Fibre;
use hopf::mesh::weave;
use hopf::obj::Obj;

fn main() -> Result<(), std::io::Error> {
    // TODO Take seed from stdIn.

    const NUM_POINTS_PER_LOOP: u32 = 80_u32;
    const NUM_TRIES: u32 = 2000_u32;
    let stdout = std::io::stdout();
    let handle = stdout.lock();
    let mut writer = BufWriter::new(handle);

    let mut meshes = vec![];

    let scale = 0.1;
    // Big outer shell.
    let start = (10_f64.to_radians(), 0_f64);
    let end = (10_f64.to_radians(), 270_f64.to_radians());
    let mesh = weave(&start, &end, 27);
    meshes.push(mesh);

    let start = (20_f64.to_radians(), 0_f64);
    let end = (20_f64.to_radians(), 270_f64.to_radians());
    let mesh = weave(&start, &end, 27);
    meshes.push(mesh);

    let start = (30_f64.to_radians(), 0_f64);
    let end = (30_f64.to_radians(), 270_f64.to_radians());
    let mesh = weave(&start, &end, 27);
    meshes.push(mesh);

    let mut obj = Obj::default();

    for (i, mesh) in meshes.into_iter().enumerate() {
        let mut seed_iter = mesh;

        // Inspect don't consume.
        let (initial_lat, initial_lon) = seed_iter
            .next()
            .expect("Must have more than one seed to make a mesh");

        let fibre_last = Fibre::new(initial_lat, initial_lon, 0_f64, 4.0 * std::f64::consts::PI);

        let (mut points_last, _alphas) = fibre_last
            .build(scale, NUM_POINTS_PER_LOOP, NUM_TRIES)
            .map_err(|_| {
                std::io::Error::other("Oscillation detected while adaptively constructing a fibre")
            })?;

        let mut quads = vec![];

        for (lat, lon) in seed_iter {
            let fibre = Fibre::new(lat, lon, 0_f64, 4.0 * std::f64::consts::PI);

            let (points, _alphas) =
                fibre
                    .build(scale, NUM_POINTS_PER_LOOP, NUM_TRIES)
                    .map_err(|_| {
                        std::io::Error::other(
                            "Oscillation detected while adaptively constructing a fibre",
                        )
                    })?;

            assert_eq!(points.len(), NUM_POINTS_PER_LOOP as usize);

            for i in 1..NUM_POINTS_PER_LOOP as usize {
                let i0 = obj.add_vertex(&points_last[i - 1]);
                let i1 = obj.add_vertex(&points_last[i]);
                let i2 = obj.add_vertex(&points[i]);
                let i3 = obj.add_vertex(&points[i - 1]);
                // Push a quad (Obj files default to anti-clockwise winding order).
                quads.push([i0, i1, i2, i3]);
            }

            points_last = points;
        }
        let name = format!("o object_{i}");
        obj.push_quads(name, quads);
    }

    obj.write_out(&mut writer)
        .map_err(|_| Error::other("Error writing output."))?;
    Ok(())
}
