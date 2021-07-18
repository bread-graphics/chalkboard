// MIT/Apache2 License

use super::{polyline, Line, Point};

#[cfg(not(feature = "std"))]
use micromath::F32Ext;

/// A bezier curve, with four control points. This allows one to represent a curve.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BezierCurve {
    /// The starting point.
    pub start: Point,
    /// The first control point.
    pub control1: Point,
    /// The second control point.
    pub control2: Point,
    /// The ending point.
    pub end: Point,
}

impl BezierCurve {
    /// Evaluate the bezier curve at a certain `t`.
    #[inline]
    fn eval_at(self, t: f32) -> Point {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        let x = (self.start.x as f32 * mt3)
            + (3.0 * self.control1.x as f32 * mt2 * t)
            + (3.0 * self.control2.x as f32 * mt * t2)
            + (self.end.x as f32 * t3);
        let y = (self.start.y as f32 * mt3)
            + (3.0 * self.control1.y as f32 * mt2 * t)
            + (3.0 * self.control2.y as f32 * mt * t2)
            + (self.end.y as f32 * t3);
        let x = x.round();
        let y = y.round();
        Point {
            x: x as i32,
            y: y as i32,
        }
    }

    /// Get the approximate number of segments we need to represent the bezier curve.
    #[inline]
    fn num_segments(self) -> usize {
        let approx_length = self.start.distance_to(self.control1)
            + self.control1.distance_to(self.control2)
            + self.control2.distance_to(self.end);
        ((approx_length.powi(2) + 800.0).sqrt() / 8.0) as usize
    }

    /// Get an iterator over the points that make up this curve.
    #[inline]
    pub fn into_points(self) -> impl Iterator<Item = Point> {
        let num_segments = self.num_segments();
        let interval = 1f32 / (num_segments as f32);

        (0..num_segments).map(move |i| {
            let t = (i as f32 + 1.0) * interval;
            self.eval_at(t)
        })
    }

    /// Get an iterator over a set of connecting lines that make up this curve.
    #[inline]
    pub fn into_lines(self) -> impl Iterator<Item = Line> {
        polyline(self.into_points())
    }
}
