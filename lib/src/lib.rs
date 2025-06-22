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

use std::fmt::Write;
/// A struct and methods for generating a Hopf fibration.
pub mod fibre;

// /// A Point Cloud.
// #[derive(Debug)]
// pub struct PointCloud(pub Vec<(f64, f64, f64)>);

/// Stereographic projection of a fibre onto the base space.
#[must_use]
#[allow(non_snake_case)]
pub fn project((X0, X1, X2, X3): (f64, f64, f64, f64)) -> (f64, f64, f64) {
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
#[must_use]
pub fn generate_ply(point_gen: impl ExactSizeIterator<Item = (f64, f64, f64)>) -> String {
    let mut ply = String::new();
    ply.push_str("ply\n");
    ply.push_str("format ascii 1.0\n");
    ply.push_str(&format!("element vertex {}\n", point_gen.len()));
    ply.push_str("property float x\n");
    ply.push_str("property float y\n");
    ply.push_str("property float z\n");
    ply.push_str("end_header\n");

    for (x, y, z) in point_gen {
        // ply.push_str(&format!("{x} {y} {z}\n"));
        writeln!(ply, "{x} {y} {z}").unwrap();
    }
    ply
}

/// Generate an OBJ file from a `PointCloud`.
#[must_use]
pub fn generate_obj(lines_gen: &[Vec<(f64, f64, f64)>]) -> String {
  let mut obj = String::new();
  for (i,line) in lines_gen.iter().enumerate() {
    obj.push_str(&format!("o fibre_{i}\n"));
    for (x, y, z) in line {
      writeln!(obj, "v {x} {y} {z}").unwrap();
    }
    obj.push_str("g hopf_fibration\n");
  }
  obj
}