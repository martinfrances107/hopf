use crate::Vertex;
use std::io::Write;
use std::{collections::HashMap, io::BufWriter};

/// Hold state information related to the storage of
/// quads in a OBJ file.
#[derive(Debug)]
pub struct Obj {
    next_index: usize,
    /// Deduping mechanism.
    pub vertex_store: HashMap<Vertex, usize>,
    /// all point that appear in a obj file.
    pub vertex_buffer: Vec<Vertex>,
    /// A list of quads keyed by object name.
    pub quad_store: HashMap<String, Vec<[usize; 4]>>,
}

impl Default for Obj {
    fn default() -> Self {
        Self {
            next_index: 1,
            vertex_store: HashMap::default(),
            vertex_buffer: Vec::new(),
            quad_store: HashMap::default(),
        }
    }
}

impl Obj {
    /// Add a point to the obj file.
    /// If the point has been seen before it will be deduplicated.
    /// and a exiting index into the vertex buffer will be returned.
    pub fn add_vertex(&mut self, p: &Vertex) -> usize {
        if let Some(v) = self.vertex_store.get(p) {
            *v
        } else {
            // first time seeing this points
            // add it to buffer and the store.
            let index = self.next_index;
            self.vertex_store.insert(p.clone(), index);
            self.vertex_buffer.push(p.clone());
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
    pub fn write_out<W>(&mut self, out: &mut BufWriter<W>) -> Result<(), std::io::Error>
    where
        W: ?Sized + std::io::Write,
    {
        // Root vertex list.
        for Vertex(x, y, z) in &self.vertex_buffer {
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
