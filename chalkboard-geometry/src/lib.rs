// MIT/Apache2 License

//! Contains definitions of several geometry primitives used in the `chalkboard` crate. Most of these primitives
//! are based around 32-bit integers, which in most cases represent pixels, or around 32-bit floating point,
//! which represent intensity on a scale from zero to one.
//!
//! Note that this crate also contains colors, but `chalkboard-geometry-and-colors` just doesn't have the same
//! ring to it.

#![no_std]
#![warn(clippy::pedantic)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod angle;
pub use angle::*;

mod curve;
pub use curve::*;

mod intensity;
pub use intensity::*;

mod path;
pub use path::*;

use core::cmp;

#[cfg(not(feature = "std"))]
use micromath::F32Ext;

/// A point in two-dimensional space. The X axis represents horizontal space, from left to right. The Y axis
/// represents vertical space, from top to bottom.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    /// X coordinate.
    pub x: i32,
    /// Y coordinate.
    pub y: i32,
}

impl Point {
    /// Get the distance from this point to another point.
    #[inline]
    pub fn distance_to(self, other: Point) -> f32 {
        ((self.x as f32 - other.x as f32).powi(2) + (self.y as f32 - other.y as f32).powi(2)).sqrt()
    }
}

/// A straight line between two points in two-dimensional space.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Line {
    /// X coordinate of the first point.
    pub x1: i32,
    /// Y coordinate of the first point.
    pub y1: i32,
    /// X coordinate of the second point.
    pub x2: i32,
    /// Y coordinate of the second point.
    pub y2: i32,
}

impl Line {
    /// The first point on this line, represented by the `x1` and `y1` fields.
    #[inline]
    #[must_use]
    pub fn point1(self) -> Point {
        Point {
            x: self.x1,
            y: self.y1,
        }
    }

    /// The second point on this line, represented by the `x2` and `y2` fields.
    #[inline]
    #[must_use]
    pub fn point2(self) -> Point {
        Point {
            x: self.x2,
            y: self.y2,
        }
    }

    /// Create a line from two points.
    #[inline]
    #[must_use]
    pub fn from_points(p1: Point, p2: Point) -> Line {
        Line {
            x1: p1.x,
            y1: p1.y,
            x2: p2.x,
            y2: p2.y,
        }
    }

    #[inline]
    fn contains_point(self, point: Point) -> bool {
        point.x <= cmp::max(self.x1, self.x2)
            && point.x >= cmp::min(self.x1, self.x2)
            && point.y <= cmp::max(self.y1, self.y2)
            && point.y >= cmp::min(self.x1, self.x2)
    }

    /// Tell if two lines intersect.
    #[inline]
    #[must_use]
    pub fn intersection(self, other: Line) -> bool {
        let (p1, p2) = (self.point1(), self.point2());
        let (p3, p4) = (other.point1(), other.point1());

        let o1 = Orientation::get(p1, p2, p3);
        let o2 = Orientation::get(p1, p2, p4);
        let o3 = Orientation::get(p3, p4, p1);
        let o4 = Orientation::get(p3, p4, p2);

        (o1 != o2 && o3 != o4)
            || (matches!(o1, Orientation::Colinear) && self.contains_point(p3))
            || (matches!(o2, Orientation::Colinear) && self.contains_point(p4))
            || (matches!(o3, Orientation::Colinear) && other.contains_point(p1))
            || (matches!(o4, Orientation::Colinear) && other.contains_point(p2))
    }
}

/// A rectangle in two dimensional space. One corner of the rectangle is at (`x1`, `y1`), and the other corner is
/// at (`x2`, `y2`).
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rectangle {
    /// X coordinate of the first point.
    pub x1: i32,
    /// Y coordinate of the first point.
    pub y1: i32,
    /// X coordinate of the second point.
    pub x2: i32,
    /// Y coordinate of the second point.
    pub y2: i32,
}

/// Convert an iterator over a series of points into an iterator over a series of lines connecting those points.
#[inline]
pub fn polyline<I: IntoIterator<Item = Point>>(points: I) -> impl Iterator<Item = Line> {
    points
        .into_iter()
        .scan(None, |last_point, current_point| {
            // returns Some(None) if this is the first point, and Some(Some(line)) for any other point
            // where line is the line between current_point and what was in last_point
            Some(
                last_point
                    .replace(current_point)
                    .map(move |last_point| Line::from_points(last_point, current_point)),
            )
        })
        .flatten()
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Orientation {
    Colinear,
    Clockwise,
    Counterclockwise,
}

impl Orientation {
    #[inline]
    fn get(p1: Point, p2: Point, p3: Point) -> Orientation {
        match ((p2.y - p1.y) * (p3.x - p1.x)) - ((p2.x - p1.x) * (p3.y - p1.y)) {
            0 => Orientation::Colinear,
            x if x > 0 => Orientation::Clockwise,
            _ => Orientation::Counterclockwise,
        }
    }
}
