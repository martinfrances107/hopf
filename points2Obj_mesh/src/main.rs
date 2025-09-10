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
use hopf::obj::Obj;

fn main() -> Result<(), std::io::Error> {
    // TODO Take seed from stdIn.
    // let mut seeds = vec![];

    let stdout = std::io::stdout();
    let handle = stdout.lock();
    let mut writer = BufWriter::new(handle);

    let mut meshes = vec![];

    // Big outer shell.
    let mut mesh = vec![];
    let lat = 10_f64.to_radians();
    (0..270).step_by(1).for_each(|i| {
        let lon = (0_f64 + 10_f64 + f64::from(i)).to_radians();
        mesh.push((lat, lon));
    });
    meshes.push(mesh);

    let mut mesh = vec![];
    let lat = 20_f64.to_radians();
    (0..270).step_by(5).for_each(|i| {
        let lon = (30_f64 + f64::from(i)).to_radians();
        mesh.push((lat, lon));
    });
    meshes.push(mesh);

    let mut mesh = vec![];
    let lat = 30_f64.to_radians();
    (0..270).step_by(10).for_each(|i| {
        let lon = (60_f64 + 10_f64 + f64::from(i)).to_radians();
        mesh.push((lat, lon));
    });
    meshes.push(mesh);

    let mut obj = Obj::default();

    for (i, mesh) in meshes.into_iter().enumerate() {
        let mut seed_iter = mesh.iter().take(20);

        // Inspect don't consume.
        let (initial_lat, initial_lon) = seed_iter
            .next()
            .expect("Must have more than one seed to make a mesh");

        let fibre_last = Fibre::new(
            *initial_lat,
            *initial_lon,
            0_f64,
            4.0 * std::f64::consts::PI,
        );
        let mut transform_last = fibre_last.projected_fibre();

        let (_, alphas) = fibre_last.build(48, 2000_u32).map_err(|_| {
            std::io::Error::other("Oscillation detected while adaptively constructing a fibre")
        })?;

        let mut quads = vec![];

        for (lat, lon) in seed_iter.clone() {
            let fibre = Fibre::new(*lat, *lon, 0_f64, 4.0 * std::f64::consts::PI);

            let transform = fibre.projected_fibre();

            for alphas in alphas.windows(2) {
                let alpha_prev = alphas[0];
                let alpha = alphas[1];

                let p0 = transform_last(alpha_prev);
                let i0 = obj.add_vertex(&p0);
                let p1 = transform_last(alpha);
                let i1 = obj.add_vertex(&p1);
                let p2 = transform(alpha);
                let i2 = obj.add_vertex(&p2);
                let p3 = transform(alpha_prev);
                let i3 = obj.add_vertex(&p3);
                // Push a quad (Obj files default to anti-clockwise winding order).
                // println!("{:#?}", [i0, i1, i2, i3]);
                quads.push([i0, i1, i2, i3]);
            }

            transform_last = transform;
        }
        let name = format!("o object_{i}");
        obj.push_quads(name, quads);
    }

    obj.write_out(&mut writer)
        .map_err(|_| Error::other("Error writing output."))?;
    Ok(())
}
