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

/// Collection of fibres woven into a mesh.
pub mod mesh;

// /// A Point Cloud
/// Handling OBJ file format.
pub mod obj;

use core::ops::Mul;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::{BufWriter, Write};

/// Hashable version of a point in E3.
#[derive(Debug, Clone)]
pub struct Vertex(pub f64, pub f64, pub f64);

impl Eq for Vertex {}
impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
            && self.1.to_bits() == other.1.to_bits()
            && self.2.to_bits() == other.2.to_bits()
    }
}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
        self.1.to_bits().hash(state);
        self.2.to_bits().hash(state);
    }
}

impl Mul<f64> for Vertex {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

/// Stereographic projection of a fibre onto the base space.
///
/// # Panics
///  If the point is at infinity or -infinity (X3 == 1)
#[must_use]
#[allow(non_snake_case)]
pub fn project(X0: f64, X1: f64, X2: f64, X3: f64) -> Vertex {
    if (1_f64 - X3).abs() < f64::EPSILON {
        // Handle the case where the point is at infinity or -infinity.
        // For now just stop.
        panic!("division by zero");
    } else {
        let x = X0 / (1_f64 - X3);
        let y = X1 / (1_f64 - X3);
        let z = X2 / (1_f64 - X3);
        Vertex(x, y, z)
    }
}

/// Generate a PLY file from a `PointCloud`.
///
/// # Errors
///   When writing to a buffer fails
pub fn generate_ply<W>(points: &[Vertex], out: &mut BufWriter<W>) -> Result<(), std::io::Error>
where
    W: ?Sized + std::io::Write,
{
    let len = points.len();
    writeln!(out, "ply")?;
    writeln!(out, "format ascii 1.0")?;
    writeln!(out, "element vertex {len}")?;
    writeln!(out, "property float x")?;
    writeln!(out, "property float y")?;
    writeln!(out, "property float z")?;
    writeln!(out, "end_header")?;

    for Vertex(x, y, z) in points {
        writeln!(out, "{x} {y} {z}")?;
    }

    Ok(())
}

/// Each fibre becomes a "line" in a OBJ file
///
///
/// # Errors
///   When writing to a buffer fails
pub fn generate_obj_lines<W>(
    lines_gen: &[Vec<Vertex>],
    out: &mut BufWriter<W>,
) -> Result<(), std::io::Error>
where
    W: ?Sized + std::io::Write,
{
    // in OBJ files the index runs to 1...=N
    let mut index = 1;
    for (i, line) in lines_gen.iter().enumerate() {
        writeln!(out, "o fibre_{i}")?;
        for Vertex(x, y, z) in line {
            writeln!(out, "v {x} {y} {z}")?;
        }
        write!(out, "l")?;

        // First point of the loop.
        let index0 = index;
        for _ in line {
            write!(out, " {index}")?;
            index += 1;
        }
        // Close the loop by appending the start of the loop to the end.
        writeln!(out, " {index0}")?;
    }
    Ok(())
}
