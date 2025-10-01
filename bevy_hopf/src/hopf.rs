use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;

use bevy::prelude::*;
use bevy_mesh::Indices;
use bevy_mesh::Mesh;
use bevy_mesh::MeshBuilder;
use bevy_mesh::Meshable;
use bevy_mesh::PrimitiveTopology;
use thiserror::Error;

use hopf::Vertex;
use hopf::fibre::Fibre;

/// An error when creating an hopf [`Mesh`] from a [`HopfMeshBuilder`].
#[derive(Clone, Copy, Debug, Error)]
pub enum HopfMeshError {
    /// When the when too many iterations were required to build a loop in the mesh.
    #[error("Cannot create an HopfMesh RETRY COUNT exceeded.")]
    NRetriesExceeded {
        /// The number of retries used.
        n_tries: u32,
        /// The latitude of the seed point.
        lat: f64,
        /// The longitude of the seed point.
        lon: f64,
    },
    /// When the start and end of the line segment are the same.
    #[error("Cannot create an HopfMesh due to invalid line specification.")]
    LineError {
        /// The start of the line segment.
        lines_start: (f64, f64),
        /// The end of the line segment.
        lines_end: (f64, f64),
    },
}

// #[derive(Clone, Copy, Debug, Reflect)]
#[derive(Clone, Debug)]
struct Hopf {
    line_start: (f64, f64),
    line_end: (f64, f64),
    n_points_per_loop: u32,
    n_loops: u32,
    n_tries: u32,
}

impl Default for Hopf {
    fn default() -> Self {
        Self {
            line_start: (45_f64.to_radians(), 0.0),
            line_end: (45_f64.to_radians(), 2_f64 * std::f64::consts::PI),
            n_points_per_loop: 100,
            n_loops: 10,
            n_tries: 2000,
        }
    }
}

/// A builder used for creating a [`Mesh`] with an [`Sphere`] shape.
// #[derive(Clone, Copy, Debug, Default, Reflect)]
// #[reflect(Default, Debug, Clone)]
#[derive(Clone, Debug)]
pub struct HopfMeshBuilder {
    /// The [`Hopf`] shape.
    hopf: Hopf,
    next_index: u32,
    /// Deduping mechanism.
    pub vertex_deduple: HashMap<Vertex, u32>,
    /// A list of point in the mesh.
    pub vertex_buffer: Vec<Vec3>,
    /// [1, 2, 3, 4, 5, 6] implies two triangles (1,2,3) and (4,5,6)
    pub triangle_store: Indices,
    /// For each entry in the vertex buffer there is a corresponding normal.
    pub normals_store: Vec<Vec3>,
}

impl HopfMeshBuilder {
    /// If the point has been seen before it will be deduplicated.
    /// and a exiting index into the vertex buffer will be returned.
    pub fn add_vertex(&mut self, p: &Vertex) -> (bool, u32) {
        if let Some(v) = self.vertex_deduple.get(p) {
            (false, *v)
        } else {
            // first time seeing this points
            // add it to buffer and the store.
            let index = self.next_index;
            self.vertex_deduple.insert(*p, index);
            self.vertex_buffer
                .push(Vec3::new(p.0.x as f32, p.0.y as f32, p.0.z as f32));
            self.next_index += 1;
            (true, index)
        }
    }

    /// Add a triangle to the mesh.
    /// The points will be de-duped and normals computed.
    pub fn add_triangle(&mut self, p0: &Vertex, p1: &Vertex, p2: &Vertex) {
        let (is_new0, i0) = self.add_vertex(p0);
        let (is_new1, i1) = self.add_vertex(p1);
        let (is_new2, i2) = self.add_vertex(p2);
        // Push the triangle ( anti-clockwise winding order ).
        self.triangle_store.push(i0);
        self.triangle_store.push(i1);
        self.triangle_store.push(i2);

        // Compute the normal vector from the cross product of it two edges.
        if is_new0 {
            let u = p1.0 - p0.0;
            let v = p2.0 - p0.0;

            let n = u.cross(v);
            let n = Vec3::new(n.x as f32, n.y as f32, n.z as f32).normalize();
            self.normals_store.push(n);
        }
        if is_new1 {
            let u = p2.0 - p1.0;
            let v = p0.0 - p1.0;
            let n = u.cross(v);
            let n = Vec3::new(n.x as f32, n.y as f32, n.z as f32).normalize();
            self.normals_store.push(n);
        }

        if is_new2 {
            let u = p0.0 - p2.0;
            let v = p1.0 - p2.0;
            let n = u.cross(v);
            let n = Vec3::new(n.x as f32, n.y as f32, n.z as f32).normalize();
            self.normals_store.push(n);
        }
    }
}

impl HopfMeshBuilder {
    /// Creates a new [`HopfMeshBuilder`].
    #[must_use]
    #[inline]
    pub fn new(
        line_start: &(f64, f64),
        line_end: &(f64, f64),
        n_points_per_loop: u32,
        n_loops: u32,
        n_tries: u32,
    ) -> Self {
        Self {
            hopf: Hopf {
                line_start: *line_start,
                line_end: *line_end,
                n_points_per_loop,
                n_loops,
                n_tries,
            },
            next_index: 1_u8.into(),
            vertex_deduple: HashMap::default(),
            vertex_buffer: Vec::new(),
            triangle_store: Indices::U32(Vec::new()),
            normals_store: Vec::new(),
        }
    }

    /// Creates an hopf mesh with the given number of subdivisions.
    ///
    /// The number of faces quadruples with each subdivision.
    /// If there are `80` or more subdivisions, the vertex count will be too large,
    /// and an [`HopfMeshError`] is returned.
    ///
    /// A good default is `5` subdivisions.
    pub fn construct(
        &mut self,
        n_loops: u32,
        n_points_per_loop: u32,
        scale: f64,
    ) -> Result<Mesh, HopfMeshError> {
        const NUM_TRIES: u32 = 2000_u32;

        // weave is a series of seed points which will be transformed into fibres.
        let line_start = self.hopf.line_start;
        let line_end = self.hopf.line_end;
        let n_loops = self.hopf.n_loops;
        let mut weave = hopf::mesh::weave(&line_start, &line_end, n_loops);

        let (initial_lat, initial_lon) = weave.next().ok_or(HopfMeshError::LineError {
            lines_start: line_start,
            lines_end: line_end,
        })?;

        let fibre_last = Fibre::new(initial_lat, initial_lon, 0_f64, 4.0 * std::f64::consts::PI);

        let (mut points_last, _alphas) = fibre_last
            // .build(NUM_POINTS_PER_LOOP, NUM_TRIES)
            .build(scale, self.hopf.n_points_per_loop, NUM_TRIES)
            .map_err(|_| HopfMeshError::NRetriesExceeded {
                n_tries: NUM_TRIES,
                lat: initial_lat,
                lon: initial_lon,
            })?;

        for (lat, lon) in weave {
            let fibre = Fibre::new(lat, lon, 0_f64, 4.0 * std::f64::consts::PI);

            let (points, _alphas) = fibre
                .build(scale, self.hopf.n_points_per_loop, NUM_TRIES)
                .map_err(|_| HopfMeshError::NRetriesExceeded {
                    n_tries: NUM_TRIES,
                    lat,
                    lon,
                })?;

            debug_assert_eq!(points.len(), self.hopf.n_points_per_loop as usize);

            //  0 - 3
            //  | / |
            //  |/  |
            //  1 --2
            //
            // Given a quad ( points 0, 1, 2, 3 )
            // form triangles (0,1,3) and (1,2,3)
            // add triangles will de-dupe points and compute normals.
            for i in 1..self.hopf.n_points_per_loop as usize {
                let p0 = &points_last[i - 1];
                let p1 = &points_last[i];
                let p2 = &points[i];
                let p3 = &points[i - 1];
                self.add_triangle(&p0.clone(), &p1.clone(), &p3.clone());
                self.add_triangle(&p1.clone(), &p2.clone(), &p3.clone());
            }

            points_last = points;
        }

        Ok(
            Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            )
            .with_inserted_indices(self.triangle_store.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.vertex_buffer.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals_store.clone()), // .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs))
        )
    }
}

impl MeshBuilder for HopfMeshBuilder {
    /// Builds a [`Mesh`] according to the configuration in `self`.
    ///
    /// # Panics
    ///
    /// Panics if the sphere is a [`SphereKind::Ico`] with a subdivision count
    /// that is greater than or equal to `80` because there will be too many vertices.
    fn build(&self) -> Mesh {
        todo!();
        //   SphereKind::Ico { subdivisions } => self.ico(subdivisions).unwrap();
        //     match self.kind {
        //         SphereKind::Ico { subdivisions } => self.ico(subdivisions).unwrap(),
        //         SphereKind::Uv { sectors, stacks } => self.uv(sectors, stacks),
        //     }
    }
}

impl Meshable for Hopf {
    type Output = HopfMeshBuilder;

    fn mesh(&self) -> Self::Output {
        HopfMeshBuilder {
            hopf: self.clone(),
            next_index: 1,
            vertex_deduple: HashMap::default(),
            vertex_buffer: Vec::new(),
            triangle_store: Indices::U32(Vec::new()),
            normals_store: Vec::new(),
        }
    }
}

impl From<Hopf> for Mesh {
    fn from(hopf: Hopf) -> Self {
        hopf.mesh().build()
    }
}
