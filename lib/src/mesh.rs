//! Collections of fibres woven into a mesh.

/// For a line segment of s2 ( as defined by two points on the globe ) divide
/// the line segment into a series of points which will be transformed into a
/// loop/fibre.
pub fn weave<'a>(
    p1: &'a (f64, f64),
    p2: &'a (f64, f64),
    n_loops: u32,
) -> impl Iterator<Item = (f64, f64)> + 'a {
    let lat_step = (p2.0 - p1.0) / f64::from(n_loops);
    let long_step = (p2.1 - p1.1) / f64::from(n_loops);

    (0..n_loops).map(move |index| {
        let i = f64::from(index);
        let lat = i.mul_add(lat_step, p1.0);
        let lon = i.mul_add(long_step, p1.1);
        (lat, lon)
    })
}
