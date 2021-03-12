// MIT/Apache2 License

use super::{points_to_polyline, Line, Point};
use std::ops::Range;

/// A bezier curve with four control points.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BezierCurve {
    pub start: Point,
    pub control1: Point,
    pub control2: Point,
    pub end: Point,
}

impl BezierCurve {
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

    #[inline]
    pub(crate) fn num_segments(&self) -> usize {
        let approx_length = self.start.distance_to(self.control1)
            + self.control1.distance_to(self.control2)
            + self.control2.distance_to(self.end);
        ((approx_length.powi(2) + 800.0).sqrt() / 8.0) as _
    }

    #[inline]
    pub fn into_lines(self) -> impl Iterator<Item = Line> {
        points_to_polyline(self.into_points())
    }

    #[inline]
    pub fn into_points(self) -> impl Iterator<Item = Point> {
        struct PointGenerator {
            curve: BezierCurve,
            interval: f32,
            inner: Range<usize>,
        }

        impl Iterator for PointGenerator {
            type Item = Point;

            #[inline]
            fn next(&mut self) -> Option<Point> {
                let i = self.inner.next()?;
                let t = (i as f32 + 1.0) * self.interval;
                Some(self.curve.eval_at(t))
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.inner.size_hint()
            }
        }

        impl ExactSizeIterator for PointGenerator {}

        let num_segments = self.num_segments();
        let interval = 1f32 / (num_segments as f32);

        PointGenerator {
            inner: 0..num_segments,
            interval,
            curve: self,
        }
    }
}
