// MIT/Apache2 License

mod angle;
mod arc;
mod curve;

pub use angle::*;
pub use arc::*;
pub use curve::*;

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
            (lo - 1, hi.map(|hi| hi - 1))
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
