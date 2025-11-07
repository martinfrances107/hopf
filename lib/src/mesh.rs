//! Collections of fibres woven into a mesh.

use super::sp::SurfacePoint;

/// For a line segment of s2 ( as defined by two points on the globe ) divide
/// the line segment into a series of points which will be transformed into a
/// loop/fibre.
pub fn weave<'a>(
    p1: &'a SurfacePoint,
    p2: &'a SurfacePoint,
    n_loops: u32,
) -> impl Iterator<Item = SurfacePoint> + 'a {
    let lat_step = (p2.lat - p1.lat) / n_loops as f32;
    let long_step = (p2.lon - p1.lon) / n_loops as f32;

    (0..n_loops).map(move |index| {
        let i = index as f32;
        let lat = i.mul_add(lat_step, p1.lat);
        let lon = i.mul_add(long_step, p1.lon);
        SurfacePoint { lat, lon }
    })
}
