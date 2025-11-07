use core::ops::Add;

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

impl SurfacePoint {
    /// Convert to Bevy's cartesian coord system
    /// Bevy uses "right-handed Y-up".
    /// X is from left to right of screen.
    /// Y is from bottom to top.
    /// Z is out of screen ( towards the viewer)
    ///
    /// The north pole ( lat = 90degrees, lon = X )  is +Y
    #[must_use]
    pub fn to_cartesian(&self, r: f32) -> Vec3 {
        let (sin_lat, cos_lat) = self.lat.sin_cos();
        let (sin_lon, cos_lon) = self.lon.sin_cos();
        Vec3 {
            x: r * cos_lat * cos_lon,
            y: r * sin_lat,
            // Should this is be negative?
            z: -r * cos_lat * sin_lon,
        }
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

#[cfg(test)]
mod test {
    use super::*;
    use glam::Vec3;

    #[test]
    // Bevy uses "right-handed Y-up" coordinate system
    // X is left to right,
    // Y bottom to top of screen
    // Z is out of screen ( towards the viewer)
    //
    // The north pole ( lat = 90degrees, lon = 0 )  is +Y
    fn compass() {
        // (input, expected)
        let cases = [
            // X axis
            (
                SurfacePoint {
                    lat: 0_f32,
                    lon: 0_f32,
                },
                Vec3 {
                    x: 1_f32,
                    y: 0_f32,
                    z: 0_f32,
                },
            ),
            // -Z axis
            (
                SurfacePoint {
                    lat: 0_f32,
                    lon: 90_f32.to_radians(),
                },
                Vec3 {
                    x: 0_f32,
                    y: 0_f32,
                    z: -1_f32,
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
            let err_vec = output - expected;
            assert!(
                err_vec.length() < 1e-6,
                "Failed: sp {sp:#?} -> {output} expected {expected:#?}"
            );
        }
    }
}
