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

use core::fmt::Error;
use std::fmt::Write;
/// A struct and methods for generating a Hopf fibration.
pub mod fibre;
/// Calculates length of path
pub mod length;
// /// A Point Cloud.

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
pub fn generate_ply(point_gen: &[(f64, f64, f64)]) -> Result<String, Error> {
    let mut ply = String::new();
    ply.push_str("ply\n");
    ply.push_str("format ascii 1.0\n");
    writeln!(ply, "element vertex {}", point_gen.len()).unwrap();
    ply.push_str("property float x\n");
    ply.push_str("property float y\n");
    ply.push_str("property float z\n");
    ply.push_str("end_header\n");

    for (x, y, z) in point_gen {
        writeln!(ply, "{x} {y} {z}")?;
    }
    Ok(ply)
}

/// Generate an OBJ file from a `PointCloud`.
///
/// # Errors
///   When writing to a buffer fails
pub fn generate_obj(lines_gen: &[Vec<(f64, f64, f64)>]) -> Result<String, Error> {
    let mut obj = String::new();
    for (i, line) in lines_gen.iter().enumerate() {
        writeln!(obj, "o fibre_{i}")?;
        for (x, y, z) in line {
            writeln!(obj, "v {x} {y} {z}")?;
        }
        obj.push_str("g hopf_fibration\n");
    }
    Ok(obj)
}
