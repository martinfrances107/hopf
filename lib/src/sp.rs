use core::ops::Add;
use core::ops::Sub;

use glam::Vec2;
use glam::Vec3;

/// Point on a Sphere.
///
/// I could use (f32, f32) or `glam::Vec` but I
/// explicit field labels lat and lon.
#[derive(Copy, Clone)]
pub struct SurfacePoint {
    /// latitude ( radians )
    pub lat: f32,
    /// longitude ( radians )
    pub lon: f32,
}

/// Interanally stored in radians, displayed in degrees
impl core::fmt::Debug for SurfacePoint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SphericalPoint")
            .field("longitude(degrees)", &self.lon.to_degrees())
            .field("latitude(degrees)", &self.lat.to_degrees())
            .finish()
    }
}

impl core::fmt::Display for SurfacePoint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // a
        write!(
            f,
            "(lat: {}, lon: {} degrees)",
            self.lat.to_degrees(),
            self.lon.to_degrees()
        )
    }
}
impl SurfacePoint {
    /// Convert to Bevy's cartesian coord system
    /// Bevy uses right-handed Y up.
    /// X is from left to right of screen.
    /// Y is from bottom to top.
    /// Z is out of screen ( towards the viewer)
    ///
    /// The north pole ( lat = 90degrees, lon = X )  is +Y
    ///
    /// The -Z axes is aligned to lat 0, long 0.
    /// So that we can use `Transform::forward()`.
    /// NB -Z points into the screen.
    ///
    /// Y - North Polt (lan 90 degree, lon X)
    /// |
    /// |  -Z .. (lat 0, lon 0)
    /// | /
    /// |/
    /// +------> X
    #[must_use]
    pub fn to_cartesian(&self, r: f32) -> Vec3 {
        let (sin_lat, cos_lat) = self.lat.sin_cos();
        let (sin_lon, cos_lon) = self.lon.sin_cos();
        Vec3 {
            x: r * cos_lat * sin_lon,
            y: r * sin_lat,
            // Negative points forward into the screen.
            z: -r * cos_lat * cos_lon,
        }
    }

    fn extract_surface_point(direction: Vec3) -> Self {
        let Vec3 { x, y, z } = direction;
        println!("direction {direction:#?}");
        // hypotenu is 1.
        let lat = f32::asin(y);
        let lon = f32::atan2(-z, x);
        Self { lat, lon }
    }
}

impl Add<Vec2> for SurfacePoint {
    type Output = Self;
    fn add(self, other: Vec2) -> Self {
        Self {
            lat: self.lat + other.x,
            lon: self.lon + other.y,
        }
    }
}

impl Add<SurfacePoint> for f32 {
    type Output = SurfacePoint;
    fn add(self, other: SurfacePoint) -> SurfacePoint {
        SurfacePoint {
            lat: self + other.lat,
            lon: self + other.lon,
        }
    }
}

impl Add<Self> for SurfacePoint {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            lat: self.lat + other.lat,
            lon: self.lon + other.lon,
        }
    }
}

impl Sub<Self> for SurfacePoint {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            lat: self.lat - other.lat,
            lon: self.lon - other.lon,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use glam::Vec3;

    #[test]
    // Bevy uses right-handed Y up coordinate system
    // X is left to right,
    // Y bottom to top of screen
    // Z is out of screen ( towards the viewer)
    //
    // The north pole ( lat = 90degrees, lon = 0 )  is +Y
    fn compass() {
        // (input, expected)
        let cases = [
            (
                //(0,0) is aligned the bevy's convept of forward --- The -Z axis .. into the screen
                SurfacePoint {
                    lat: 0_f32,
                    lon: 0_f32,
                },
                Vec3 {
                    x: 0_f32,
                    y: 0_f32,
                    z: -1_f32,
                },
            ),
            // Rotate 90 long, rotates the the +X axis.
            (
                SurfacePoint {
                    lat: 0_f32,
                    lon: 90_f32.to_radians(),
                },
                Vec3 {
                    x: 1_f32,
                    y: 0_f32,
                    z: 0_f32,
                },
            ),
            // Y -- North pole
            (
                SurfacePoint {
                    lat: 90_f32.to_radians(),
                    lon: 0_f32,
                },
                Vec3 {
                    x: 0_f32,
                    y: 1_f32,
                    z: 0_f32,
                },
            ),
        ];

        for (sp, expected) in cases {
            let output = sp.to_cartesian(1_f32);

            assert!(
                output.abs_diff_eq(expected, 1e-6),
                "Failed: sp {sp:#?} -> {output} expected {expected:#?}"
            );
        }
    }
}
