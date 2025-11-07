//! Collections of fibres woven into a mesh.

use super::sp::SurfacePoint;

/// For a line segment of s2 ( as defined by two points on the globe ) divide
/// the line segment into a series of points which will be transformed into a
/// loop/fibre.
///
/// `n_loops` is the number of loop that form the mesh
///  - Limited to u16 ONLY for convience in casting.
///  - u32 cannot be easily cast to a f32.
pub fn weave<'a>(
    p1: &'a SurfacePoint,
    p2: &'a SurfacePoint,
    n_loops: u16,
) -> impl Iterator<Item = SurfacePoint> + 'a {
    let lat_step = (p2.lat - p1.lat) / f32::from(n_loops);
    let long_step = (p2.lon - p1.lon) / f32::from(n_loops);

    (0..n_loops).map(move |index| {
        let i = f32::from(index);
        let lat = i.mul_add(lat_step, p1.lat);
        let lon = i.mul_add(long_step, p1.lon);
        SurfacePoint { lat, lon }
    })
}
