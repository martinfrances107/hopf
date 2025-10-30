use crate::Vertex;
use std::io::Write;
use std::{collections::HashMap, io::BufWriter};

use glam::DVec3;

/// Hold state information related to the storage of
/// quads in a OBJ file.
#[derive(Debug)]
pub struct Obj {
    next_index: usize,
    /// A deduplicated list of points which
    /// will be sorted and copied into a vertex buffer.
    pub vertex_store: HashMap<Vertex, usize>,
    // /// all point that appear in a obj file.
    // pub vertex_buffer: Vec<Vertex>,
    /// A list of quads keyed by object name.
    pub quad_store: HashMap<String, Vec<[usize; 4]>>,
}

impl Default for Obj {
    fn default() -> Self {
        Self {
            // wavefront Obj file start at index 1.
            next_index: 1,
            vertex_store: HashMap::default(),
            quad_store: HashMap::default(),
        }
    }
}

impl Obj {
    /// Add a point to the obj file.
    /// If the point has been seen before it will be deduplicated.
    /// and the exiting index into the vertex buffer will be returned.
    pub fn add_vertex(&mut self, p: &Vertex) -> usize {
        if let Some(v) = self.vertex_store.get(p) {
            *v
        } else {
            // first time seeing this points
            // add it to buffer and the store.
            let index = self.next_index;
            self.vertex_store.insert(*p, index);
            self.next_index += 1;
            index
        }
    }

    /// Push a prepared list of quads into the OBJ.
    pub fn push_quads(&mut self, name: String, quads: Vec<[usize; 4]>) {
        self.quad_store.insert(name, quads);
    }

    /// Writes `vertex_buffer` and quad information out to file.
    ///
    /// # Errors
    ///   When writing to a buffer fails
    ///
    /// # Panics
    ///   When a vertex written to the store, cannot be read.
    pub fn write<W>(mut self, out: &mut BufWriter<W>) -> Result<(), std::io::Error>
    where
        W: ?Sized + std::io::Write,
    {
        let mut vertex_buffer = self.vertex_store.drain().collect::<Vec<_>>();
        vertex_buffer.sort_by(|a, b| {
            // sort by value
            a.1.cmp(&b.1)
        });

        // Root vertex list.
        for (Vertex(DVec3 { x, y, z }), _) in vertex_buffer {
            writeln!(out, "v {x} {y} {z}")?;
        }

        // In OBJ files the index runs to 1...=N
        for (name, quads) in &self.quad_store {
            writeln!(out, "o {name}")?;
            // First point of the loop.
            for quad in quads {
                writeln!(out, "f {} {} {} {}", quad[0], quad[1], quad[2], quad[3])?;
            }
        }

        Ok(())
    }
}
