use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::Vec3;
use bevy_mesh::Indices;
use bevy_mesh::Mesh;
use bevy_mesh::MeshBuilder;
use bevy_mesh::Meshable;
use bevy_mesh::PrimitiveTopology;
use thiserror::Error;

use hopf::Vertex;
use hopf::fibre::Fibre;
use hopf::sp::SurfacePoint;

/// An error when creating an hopf [`Mesh`] from a [`HopfMeshBuilder`].
#[derive(Clone, Copy, Debug, Error)]
pub enum HopfMeshError {
    /// When the when too many iterations were required to build a loop in the mesh.
    #[error("Cannot create an HopfMesh RETRY COUNT exceeded.")]
    NRetriesExceeded {
        /// The number of retries used.
        n_tries: u16,
        /// The current point being used to generate the mesh.
        sp: SurfacePoint,
    },
    /// When the start and end of the line segment are the same.
    #[error("Cannot create an HopfMesh due to invalid line specification.")]
    LineError {
        /// The start of the line segment.
        lines_start: SurfacePoint,
        /// The end of the line segment.
        lines_end: SurfacePoint,
    },
}

// #[derive(Clone, Copy, Debug, Reflect)]
#[derive(Clone, Debug)]
struct Hopf {
    line_start: SurfacePoint,
    line_end: SurfacePoint,
    n_loops: u16,
}

impl Default for Hopf {
    fn default() -> Self {
        Self {
            line_start: SurfacePoint {
                lat: 45_f32.to_radians(),
                lon: 0.0,
            },
            line_end: SurfacePoint {
                lat: 45_f32.to_radians(),
                lon: 2_f32 * core::f32::consts::PI,
            },
            n_loops: 10,
        }
    }
}

/// A builder used for creating a [`Mesh`] with an [`Sphere`] shape.
// #[derive(Clone, Copy, Debug, Default, Reflect)]
// #[reflect(Default, Debug, Clone)]
#[derive(Clone, Debug)]
pub struct HopfMeshBuilder {
    /// Deduping mechanism.
    pub vertex_store: HashMap<Vertex, u32>,
    /// [1, 2, 3, 4, 5, 6] implies two triangles (1,2,3) and (4,5,6)
    pub triangle_store: Indices,
    /// Per vertex UVs.
    pub uv_store: Vec<[f32; 2]>,
    /// Number of tries when building a individual loop.
    pub n_tries: u16,

    // The last [`Hopf`] shape constructed.
    hopf: Hopf,
    next_index: u32,
}

impl HopfMeshBuilder {
    /// If the point has been seen before it will be deduplicated
    /// and a exiting vertex buffer index will be returned.
    pub fn add_vertex(&mut self, p: &Vertex) -> u32 {
        if let Some(index) = self.vertex_store.get(p) {
            *index
        } else {
            // first time seeing this points
            // add it to buffer and the store.
            let index = self.next_index;
            self.vertex_store.insert(*p, index);
            // self.vertex_buffer
            //     .push(Vec3::new(p.0.x as f32, p.0.y as f32, p.0.z as f32));
            // Could scale a x, y value into  uv space.
            self.uv_store.push([0.5, 0.5]); // Placeholder for UVs
            self.next_index += 1;
            index
        }
    }

    /// Add a triangle to the mesh.
    /// The points will be de-duped and normals computed.
    pub fn add_triangle(&mut self, p0: &Vertex, p1: &Vertex, p2: &Vertex) {
        let i0 = self.add_vertex(p0);
        let i1 = self.add_vertex(p1);
        let i2 = self.add_vertex(p2);
        // Push the triangle ( anti-clockwise winding order ).
        self.triangle_store.push(i0);
        self.triangle_store.push(i1);
        self.triangle_store.push(i2);
    }
}

impl HopfMeshBuilder {
    /// Creates a new [`HopfMeshBuilder`].
    #[must_use = "Not using the returned, is the same a doing nothing at all."]
    #[inline]
    pub fn new(
        line_start: &SurfacePoint,
        line_end: &SurfacePoint,
        n_loops: u16,
        n_tries: u16,
    ) -> Self {
        Self {
            hopf: Hopf {
                line_start: *line_start,
                line_end: *line_end,
                n_loops,
            },
            // Unlike Wavefront OBJ files indexed start at zero
            next_index: 0,
            vertex_store: HashMap::default(),
            triangle_store: Indices::U32(Vec::new()),
            uv_store: Vec::new(),
            n_tries,
        }
    }

    /// Creates an hopf mesh with N points per loop
    ///
    /// This logic could be folded into `HopfBuilder::build()` but build cannot fail.
    /// and I want better error reporting.
    ///
    /// # Errors
    ///
    /// `HopfMeshError::LineError` if  `line_start` and `line_end` are identical.
    ///
    /// `HopfMeshError::NRetriesExceeded` if any loop cannot be constructed.
    pub fn construct<const N: usize>(mut self) -> Result<Self, HopfMeshError> {
        // weave is a series of seed points which will be transformed into fibres.
        let line_start = self.hopf.line_start;
        let line_end = self.hopf.line_end;
        let n_loops = self.hopf.n_loops;
        let mut weave = hopf::mesh::weave(&line_start, &line_end, n_loops);

        let sp_initial = weave.next().ok_or(HopfMeshError::LineError {
            lines_start: line_start,
            lines_end: line_end,
        })?;

        let fibre_last = Fibre::new(sp_initial, 0_f32..=4.0 * core::f32::consts::PI);

        // let (mut points_last, _alphas) = fibre_last
        //     .build(self.scale, N as u32, self.n_tries)
        //     .map_err(|_| HopfMeshError::NRetriesExceeded {
        //         n_tries: self.n_tries,
        //         lat: initial_lat,
        //         lon: initial_lon,
        //     })?;

        let (mut points_last, _alphas) = fibre_last.build_uniform::<N>();

        for sp in weave {
            let fibre = Fibre::new(sp, 0_f32..=4.0 * core::f32::consts::PI);

            // let (points, _alphas) = fibre
            //     .build(self.scale, self.hopf.n_points_per_loop, self.n_tries)
            //     .map_err(|_| HopfMeshError::NRetriesExceeded {
            //         n_tries: self.n_tries,
            //         lat,
            //         lon,
            //     })?;
            // debug_assert_eq!(points.len(), 80);

            let (points, _alphas) = fibre.build_uniform::<N>();

            //  0 - 3
            //  | / |
            //  |/  |
            //  1 --2
            //
            // Given a quad ( points 0, 1, 2, 3 )
            // form triangles (0,1,3) and (1,2,3)
            // add triangles will de-dupe points and compute normals.
            for i in 1..N {
                let p0 = &points_last[i - 1];
                let p1 = &points_last[i];
                let p2 = &points[i];
                let p3 = &points[i - 1];
                self.add_triangle(&p0.clone(), &p1.clone(), &p3.clone());
                self.add_triangle(&p1.clone(), &p2.clone(), &p3.clone());
            }

            points_last = points;
        }

        Ok(self)
    }
}

impl MeshBuilder for HopfMeshBuilder {
    /// Builds a [`Mesh`] according to the configuration in `self`.
    fn build(&self) -> Mesh {
        // Construct a vertex buffer from our deduplicating hash structure
        //
        // Three loop over each vertex!  - Is there are better way?
        // 1) Conversion of hash into a vector
        // 2) Sorting the vector
        // 3) Downcasting into DVec3 into Vec3.
        let mut keyed_vertex_buffer = self.vertex_store.iter().collect::<Vec<_>>();
        keyed_vertex_buffer.sort_by(|a, b| a.1.cmp(b.1));

        let vertex_buffer: Vec<Vec3> = keyed_vertex_buffer
            .iter()
            .map(|(point, _)| (**point).into())
            .collect();

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        // .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uv_store.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertex_buffer)
        .with_inserted_indices(self.triangle_store.clone());

        mesh.duplicate_vertices();
        mesh.compute_flat_normals();

        // To get a broken version of smaooth normals.
        // Remove duplicate_verticies() (    above )  and add this line
        // mesh.compute_smooth_normals();
        mesh
    }
}

impl Meshable for Hopf {
    type Output = HopfMeshBuilder;

    fn mesh(&self) -> Self::Output {
        HopfMeshBuilder {
            hopf: self.clone(),
            next_index: 0,
            vertex_store: HashMap::default(),
            triangle_store: Indices::U32(Vec::new()),
            uv_store: Vec::new(),
            n_tries: 2000,
        }
    }
}

impl From<Hopf> for Mesh {
    fn from(hopf: Hopf) -> Self {
        hopf.mesh().build()
    }
}
