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

/// A struct and methods for generating a Hopf fibration.
pub mod fibre;
/// Calculates length of path
pub mod length;
// /// A Point Cloud.

use std::io::{LineWriter, Write};

/// Stereographic projection of a fibre onto the base space.
#[must_use]
#[allow(non_snake_case)]
pub fn project(X0: f64, X1: f64, X2: f64, X3: f64) -> (f64, f64, f64) {
    if (1_f64 - X3).abs() < f64::EPSILON {
        // Handle the case where the point is at infinity.
        (f64::NAN, f64::NAN, f64::NAN)
    } else {
        let x = X0 / (1_f64 - X3);
        let y = X1 / (1_f64 - X3);
        let z = X2 / (1_f64 - X3);
        (x, y, z)
    }
}

/// Generate a PLY file from a `PointCloud`.
///
/// # Errors
///   When writing to a buffer fails
pub fn generate_ply<W>(points: &[(f64, f64, f64)], out: &mut LineWriter<W>) -> Result<(), std::io::Error>
where
 W: ?Sized + std::io::Write
 {
  let len = points.len();
    writeln!(out, "ply")?;
    writeln!(out, "format ascii 1.0")?;
    writeln!(out, "element vertex {len}")?;
    writeln!(out, "property float x")?;
    writeln!(out, "property float y")?;
    writeln!(out, "property float z")?;
    writeln!(out, "end_header")?;

    for (x, y, z) in points {
        writeln!(out, "{x} {y} {z}")?;
    }

    Ok(())
}

/// Generate an OBJ file from a `PointCloud`.
///
/// # Errors
///   When writing to a buffer fails
pub fn generate_obj<W>(lines_gen: &[Vec<(f64, f64, f64)>], out: &mut LineWriter<W>) -> Result<(), std::io::Error>
where
 W: ?Sized + std::io::Write
{
    // in OBJ files the index runs to 1...=N
    let mut index = 1;
    for (i, line) in lines_gen.iter().enumerate() {
        writeln!(out, "o fibre_{i}")?;
        for (x, y, z) in line {
            writeln!(out, "v {x} {y} {z}")?;
        }
        // out.push_str("g hopf_fibration\n");
        write!(out, "l")?;

        // First point of the loop.
        let index0 = index;
        for _ in line{
          write!(out, " {index}")?;
          index += 1;
        }
        // Close the loop by appending the start of the loop to the end.
        writeln!(out, " {index0}")?;

    }
    Ok(())
}
