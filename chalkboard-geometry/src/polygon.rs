// BSL 1.0 License

use super::{Line, Point2D, Scalar, PathEvent};
use alloc::vec::Vec;
use lyon_path::{iterator::PathIterator, PathBufferSlice, PathBuffer, PathSlice, Path};
use num_traits::Zero;
use tinyvec::TinyVec;

const MAX_POLYGON_STACK_SIZE: usize = 32;

/// One or more closed polygons.
///
/// Many types in this crate are able to be simplified into this type,
/// which may be easier to work with.
#[derive(Debug, Default)]
pub struct Polygon {
    /// The edges for this polygon.
    edges: Vec<Edge<f32>>,
}

type Event = PathEvent<Point2D<f32>, Point2D<f32>>;

impl Polygon {
    /// Add an edge to this polygon.
    pub fn add_edge(&mut self, p1: Point2D<f32>, p2: Point2D<f32>) {
        let edge = Edge::new(p1, p2);

        // don't add perfectly horizontal edges
        if edge.line.vector.y.abs() >= f32::EPSILON {
            self.edges.push(edge);
        }
    }

    /// Collect from a path event iterator with a given tolerance.
    pub fn from_iter_with_tolerance(
        iter: impl IntoIterator<Item = Event>,
        tolerance: f32,
    ) -> Self {
        iter.into_iter().flattened(tolerance)
            .filter_map(|event| match event {
                PathEvent::Begin { .. } => {
                    None
                },
                PathEvent::Line { from, to } => {
                    Some(Edge::new(from, to))
                },
                PathEvent::End { last, first, close } => {
                    Some(Edge::new(last, first))
                },
                ev => unreachable!(
                    "Flattened iterator should never yield {:?}",
                    ev,
                ),
            }).collect()
    }
}

impl IntoIterator for Polygon {
    type Item = Edge<f32>;
    type IntoIter = alloc::vec::IntoIter<Edge<f32>>;

    fn into_iter(self) -> Self::IntoIter {
        self.edges.into_iter()
    }
}

impl FromIterator<Edge<f32>> for Polygon {
    fn from_iter<T: IntoIterator<Item = Edge<f32>>>(iter: T) -> Self {
        Self {
            edges: Vec::from_iter(iter),
        }
    }
}

impl FromIterator<Event> for Polygon {
    fn from_iter<T: IntoIterator<Item = PathEvent<Point2D<f32>, Point2D<f32>>>>(iter: T) -> Self {
        const DEFAULT_TOLERANCE: f32 = 2.0;
        Self::from_iter_with_tolerance(iter, DEFAULT_TOLERANCE)
    }
}

impl<'a> From<PathBufferSlice<'a>> for Polygon {
    fn from(pbs: PathBufferSlice<'a>) -> Self {
        pbs.iter().flat_map(|i| i.iter()).collect()
    }
}

impl From<PathBuffer> for Polygon {
    fn from(pb: PathBuffer) -> Self {
        pb.as_slice().into()
    }
}

impl<'a> From<PathSlice<'a>> for Polygon {
    fn from(ps: PathSlice<'a>) -> Self {
        ps.iter().collect() 
    }
}

impl From<Path> for Polygon {
    fn from(p: Path) -> Self {
        p.as_slice().into()
    }
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

impl<Num: Scalar> Edge<Num> {
    /// Create a new `Edge` from two points.
    pub fn new(p1: Point2D<Num>, p2: Point2D<Num>) -> Self {
        let line = Line {
            point: p1,
            vector: p2 - p1,
        };

        let (top, bottom) = if p1.y < p2.y {
            (p1.y, p2.y)
        } else {
            (p2.y, p1.y)
        };

        Self {
            line,
            top,
            bottom,
            direction: Direction::Forward,
        }
    }

    /// Get the intersection of two edges, if any.
    pub fn intersection(&self, other: &Edge<Num>) -> Option<Point2D<Num>> {
        let inter = self.line.intersection(&other.line)?;
        if inter.y >= self.top
            && inter.y <= self.bottom
            && inter.y >= other.top
            && inter.y <= other.bottom
        {
            Some(inter)
        } else {
            None
        }
    }
}

/// The direction that an `Edge` moves in.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Direction {
    #[default]
    Forward,
    Backwards,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersection() {
        // two edges that do intersect
        let e1 = Edge::new(Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0));
        let e2 = Edge::new(Point2D::new(1.0, 0.0), Point2D::new(0.0, 1.0));
        assert_eq!(e1.intersection(&e2), Some(Point2D::new(0.5, 0.5)));

        // two edges that do not intersect
        let e1 = Edge::new(Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0));
        let e2 = Edge::new(Point2D::new(1.0, 0.0), Point2D::new(2.0, 1.0));
        assert_eq!(e1.intersection(&e2), None);

        // two edges that are not parallel lines but do not intersect
        let e1 = Edge::new(Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0));
        let e2 = Edge::new(Point2D::new(1.0, 0.0), Point2D::new(2.0, 2.0));
        assert_eq!(e1.intersection(&e2), None);
    }
}
