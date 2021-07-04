// MIT/Apache2 License

use std::{cmp, iter::FusedIterator, mem};

mod angle;
mod arc;
mod curve;

pub use angle::*;
pub use arc::*;
pub use curve::*;

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

/// A point in two-dimensional space.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    #[inline]
    pub(crate) fn distance_to(self, other: Point) -> f32 {
        ((self.x as f32 - other.x as f32).powi(2) + (self.y as f32 - other.y as f32).powi(2)).sqrt()
    }
}

/// A line between two points.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Line {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

impl Line {
    #[inline]
    pub fn point1(self) -> Point {
        Point {
            x: self.x1,
            y: self.y1,
        }
    }

    #[inline]
    pub fn point2(self) -> Point {
        Point {
            x: self.x2,
            y: self.y2,
        }
    }

    #[inline]
    pub fn from_points(p1: Point, p2: Point) -> Self {
        let Point { x: x1, y: y1 } = p1;
        let Point { x: x2, y: y2 } = p2;
        Self { x1, y1, x2, y2 }
    }

    #[inline]
    pub fn contains_point(self, point: Point) -> bool {
        point.x <= cmp::max(self.x1, self.x2)
            && point.x >= cmp::min(self.x1, self.x2)
            && point.y <= cmp::max(self.y1, self.y2)
            && point.y >= cmp::min(self.x1, self.x2)
    }

    #[inline]
    pub(crate) fn intersection(self, other: Self) -> bool {
        // derived from https://www.geeksforgeeks.org/check-if-two-given-line-segments-intersect/
        let (p1, p2) = (self.point1(), self.point2());
        let (p3, p4) = (other.point1(), other.point2());

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

    #[inline]
    pub(crate) fn point_swap(&mut self) {
        mem::swap(&mut self.x1, &mut self.x2);
        mem::swap(&mut self.y1, &mut self.y2);
    }
}

/// A rectangle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rectangle {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

/// Convert a series of points to a series of connected lines.
#[inline]
pub(crate) fn points_to_polyline<I: IntoIterator<Item = Point>>(
    pts: I,
) -> impl Iterator<Item = Line> {
    struct Windows<T, I> {
        iter: I,
        prev: Option<T>,
    }

    impl<T: Clone, I: Iterator<Item = T>> Iterator for Windows<T, I> {
        type Item = (T, T);

        #[inline]
        fn next(&mut self) -> Option<(T, T)> {
            loop {
                match self.prev.take() {
                    None => {
                        self.prev = Some(self.iter.next()?);
                    }
                    Some(prev) => {
                        let cur = self.iter.next()?;
                        self.prev = Some(cur.clone());
                        return Some((prev, cur));
                    }
                }
            }
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            let (lo, hi) = self.iter.size_hint();
            (lo.saturating_sub(1), hi.map(|hi| hi.saturating_sub(1)))
        }
    }

    impl<T: Clone, I: Iterator<Item = T> + ExactSizeIterator> ExactSizeIterator for Windows<T, I> {}
    impl<T: Clone, I: Iterator<Item = T> + FusedIterator> FusedIterator for Windows<T, I> {}

    Windows {
        iter: pts.into_iter(),
        prev: None,
    }
    .map(|(Point { x: x1, y: y1 }, Point { x: x2, y: y2 })| Line { x1, y1, x2, y2 })
}
