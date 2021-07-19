// MIT/Apache2 License

use super::{polyline, Angle, Line, Point};
use alloc::vec::Vec;
use core::iter::{FusedIterator, Peekable};

/// Given an iterator over a set of points that forms a polyline, returns a set of points representing a closed
/// path over the outline of the polyline given the line width.
#[cfg(feature = "alloc")]
#[inline]
pub fn outline<I: IntoIterator<Item = Point>>(
    points: I,
    line_width: u32,
) -> impl Iterator<Item = Point> {
    // TODO: optimize to reduce heap allocation
    let points: Vec<Point> = points.into_iter().collect();
    let points = points.into_iter();
    let points2 = points.clone();
    let line_width = line_width as i32;

    outline_inner(points, line_width, 1).chain(outline_inner(points2.rev(), line_width, -1))
}

#[inline]
fn outline_inner<I: IntoIterator<Item = Point>>(
    points: I,
    line_width: i32,
    multiplier: i32,
) -> impl Iterator<Item = Point> {
    // TODO: a custom iterator may not be necessary here
    struct Outliner<I: Iterator> {
        lines: Peekable<I>,
        is_first_line: bool,
        line_width: i32,
        multiplier: i32,
    }

    impl<I: Iterator<Item = Line>> Iterator for Outliner<I> {
        type Item = Point;

        #[inline]
        fn next(&mut self) -> Option<Point> {
            if self.is_first_line {
                self.is_first_line = false;
                let line = *self.lines.peek()?;
                Some(line.point1() + (point_change(line, self.line_width) * self.multiplier))
            } else {
                let line = self.lines.next()?;
                Some(line.point2() + (point_change(line, self.line_width) * self.multiplier))
            }
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            let (mut lower, mut upper) = self.lines.size_hint();
            if self.is_first_line {
                lower += 1;
                upper = upper.map(|upper| upper + 1);
            }
            (lower, upper)
        }
    }

    impl<I: Iterator<Item = Line> + FusedIterator> FusedIterator for Outliner<I> {}
    impl<I: Iterator<Item = Line> + ExactSizeIterator> ExactSizeIterator for Outliner<I> {}

    Outliner {
        lines: polyline(points).peekable(),
        is_first_line: true,
        line_width,
        multiplier,
    }
}

#[inline]
fn point_change(line: Line, lw: i32) -> Point {
    let angle = line.angle().radians();
    let angle = angle + Angle::QUARTER_CIRCLE.radians();
    Point {
        x: (angle.cos() * lw as f32) as i32,
        y: (angle.sin() * lw as f32) as i32,
    }
}
