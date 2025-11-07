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

/// Surface point.
pub mod sp;

// /// A Point Cloud
/// Handling OBJ file format.
pub mod obj;

use std::hash::Hash;
use std::hash::Hasher;
use std::io::{BufWriter, Write};
use std::ops::Mul;
use std::ops::Sub;

use bytemuck::{Pod, Zeroable};
use glam::Vec3;

/// Hashable version of a point in E3.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct Vertex(pub Vec3);

impl Eq for Vertex {}
impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.0.x.to_bits() == other.0.x.to_bits()
            && self.0.y.to_bits() == other.0.y.to_bits()
            && self.0.z.to_bits() == other.0.z.to_bits()
    }
}

#[allow(clippy::cast_possible_truncation)]
impl From<Vertex> for Vec3 {
    fn from(v: Vertex) -> Self {
        let x = v.0.x;
        let y = v.0.y;
        let z = v.0.z;
        Self::new(x, y, z)
    }
}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.x.to_bits().hash(state);
        self.0.y.to_bits().hash(state);
        self.0.z.to_bits().hash(state);
    }
}

impl Sub<Self> for Vertex {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(Vec3 {
            x: self.0.x - rhs.0.x,
            y: self.0.y - rhs.0.y,
            z: self.0.z - rhs.0.z,
        })
    }
}

impl Mul<f32> for Vertex {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(Vec3 {
            x: self.0.x * rhs,
            y: self.0.y * rhs,
            z: self.0.z * rhs,
        })
    }
}

impl Vertex {
    /// Computes the dot product of `self` and `rhs`.
    #[inline]
    #[must_use]
    pub fn dot(self, rhs: Self) -> f32 {
        // (self.0.x * rhs.0.x) + (self.0.y * rhs.0.y) + (self.0.z * rhs.0.z)
        self.0
            .z
            .mul_add(rhs.0.z, self.0.x.mul_add(rhs.0.x, self.0.y * rhs.0.y))
    }

    /// Computes the length of `self`.
    #[doc(alias = "magnitude")]
    #[inline]
    #[must_use]
    pub fn length(self) -> f32 {
        let d = self.dot(self);
        d.sqrt()
    }
}

/// Stereographic projection of a fibre onto the base space.
///
/// # Panics
///  If the point is at infinity or -infinity (X3 == 1)
#[must_use = "Not using the returned, will drop the computation."]
#[allow(non_snake_case)]
pub fn project(X0: f32, X1: f32, X2: f32, X3: f32) -> Vertex {
    if (1_f32 - X3).abs() < f32::EPSILON {
        // Handle the case where the point is at infinity or -infinity.
        // For now just stop.
        panic!("division by zero");
    } else {
        let x = X0 / (1_f32 - X3);
        let y = X1 / (1_f32 - X3);
        let z = X2 / (1_f32 - X3);
        Vertex(Vec3 { x, y, z })
    }
}

/// Generate a PLY file from a `PointCloud`.
///
/// # Errors
///   When writing to a buffer fails
pub fn generate_ply<I, W>(points: I, out: &mut BufWriter<W>) -> Result<(), std::io::Error>
where
    I: ExactSizeIterator<Item = Vertex>,
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

    for Vertex(Vec3 { x, y, z }) in points {
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
        for Vertex(Vec3 { x, y, z }) in line {
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
