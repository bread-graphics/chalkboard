// MIT/Apache2 License

use super::Point;

/// A vertex that we output to the OpenGL driver.
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct Vertex {
    pub pos: Point,
    pub clr: Point,
}
