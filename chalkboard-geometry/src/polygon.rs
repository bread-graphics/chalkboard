// BSL 1.0 License

use super::Line;
use alloc::vec::Vec;
use tinyvec::TinyVec;

const MAX_POLYGON_STACK_SIZE: usize = 32;

/// One or more closed polygons.
///
/// Many types in this crate are able to be simplified into this type,
/// which may be easier to work with.
pub struct Polygon {
    /// The edges for this polygon.
    edges: Vec<Edge<f32>>,
}

/// An edge in a `Polygon`.
#[derive(Debug, Copy, Clone)]
pub struct Edge<Num> {
    /// The line that this edge exists along.
    pub line: Line<Num>,
    /// The highest point (lowest Y) on this edge.
    pub top: Num,
    /// The lowest point (highest Y) on this edge.
    pub bottom: Num,
    /// The direction this edge goes in.
    pub direction: Direction,
}

/// The direction that an `Edge` moves in.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Direction {
    #[default]
    Forward,
    Backwards,
}
