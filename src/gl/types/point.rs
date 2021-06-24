// MIT/Apache2 License

/// A three-dimensional point in OpenGL.
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct GlPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl GlPoint {
    #[inline]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl From<(f32, f32, f32)> for GlPoint {
    #[inline]
    fn from((x, y, z): (f32, f32, f32)) -> Self {
        Self { x, y, z }
    }
}
