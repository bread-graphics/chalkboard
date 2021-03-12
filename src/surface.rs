// MIT/Apache2 License

use crate::{
    color::Color,
    geometry::{Angle, BezierCurve, GeometricArc, Line, Rectangle},
    path::{Path, PathSegment, PathSegmentType},
};
use std::{array::IntoIter as ArrayIter, iter};

/// A surface which drawing commands can be applied to.
pub trait Surface {
    /// Set the color used to draw lines.
    fn set_stroke_color(&mut self, color: Color) -> crate::Result;
    /// Set the color used to fill shapes.
    fn set_fill_color(&mut self, color: Color) -> crate::Result;
    /// Set the width used to draw lines.
    fn set_line_width(&mut self, width: usize) -> crate::Result;

    /// Flush all commands passed to this surface to its target.
    fn flush(&mut self) -> crate::Result;

    /// Draw a single line.
    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result;
    /// Draw several lines. In many cases this is more efficient than drawing a single line in a loop.
    #[inline]
    fn draw_lines(&mut self, lines: &[Line]) -> crate::Result {
        lines
            .iter()
            .copied()
            .try_for_each(|Line { x1, y1, x2, y2 }| self.draw_line(x1, y1, x2, y2))
    }

    /// Draw a path.
    #[inline]
    fn draw_path(&mut self, path: Path) -> crate::Result {
        let lines = path.into_lines();
        self.draw_lines(&lines)
    }

    /// Draw a bezier curve.
    #[inline]
    fn draw_bezier_curve(&mut self, curve: BezierCurve) -> crate::Result {
        let lines: Vec<Line> = curve.into_lines().collect();
        self.draw_lines(&lines)
    }

    /// Draw several bezier curves. In many cases this is more efficient than drawing a single curve in a loop.
    #[inline]
    fn draw_bezier_curves(&mut self, curves: &[BezierCurve]) -> crate::Result {
        let lines: Vec<Line> = curves
            .iter()
            .copied()
            .flat_map(|curve| curve.into_lines())
            .collect();
        self.draw_lines(&lines)
    }

    /// Draw a rectangle.
    #[inline]
    fn draw_rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        let lines: [Line; 4] = [
            Line { x1, y1, x2, y2: y1 },
            Line { x1: x2, y1, x2, y2 },
            Line { x1, y1: y2, x2, y2 },
            Line { x1, y1, x2: x1, y2 },
        ];

        self.draw_lines(&lines)
    }

    /// Draw several rectangles. In many cases this is more efficient than drawing a single rectangle in a loop.
    #[inline]
    fn draw_rectangles(&mut self, rects: &[Rectangle]) -> crate::Result {
        let lines: Vec<Line> = rects
            .iter()
            .copied()
            .flat_map(|Rectangle { x1, y1, x2, y2 }| {
                ArrayIter::new([
                    Line { x1, y1, x2, y2: y1 },
                    Line { x1: x2, y1, x2, y2 },
                    Line { x1, y1: y2, x2, y2 },
                    Line { x1, y1, x2: x1, y2 },
                ])
            })
            .collect();
        self.draw_lines(&lines)
    }

    /// Draw an arc.
    #[inline]
    fn draw_arc(
        &mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        start: Angle,
        end: Angle,
    ) -> crate::Result {
        let lines: Vec<Line> = GeometricArc {
            x1,
            y1,
            x2,
            y2,
            start,
            end,
        }
        .into_lines();
        self.draw_lines(&lines)
    }

    /// Draw several arcs.
    #[inline]
    fn draw_arcs(&mut self, arcs: &[GeometricArc]) -> crate::Result {
        let lines: Vec<Line> = arcs
            .iter()
            .copied()
            .flat_map(|arc| arc.into_lines())
            .collect();
        self.draw_lines(&lines)
    }

    /// Draw an ellipse.
    #[inline]
    fn draw_ellipse(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        self.draw_arc(x1, y1, x2, y1, Angle::ZERO, Angle::FULL_CIRCLE)
    }

    /// Draw several ellipses.
    #[inline]
    fn draw_ellipses(&mut self, rects: &[Rectangle]) -> crate::Result {
        let arcs: Vec<GeometricArc> = rects
            .iter()
            .copied()
            .map(|Rectangle { x1, y1, x2, y2 }| GeometricArc {
                x1,
                y1,
                x2,
                y2,
                start: Angle::ZERO,
                end: Angle::FULL_CIRCLE,
            })
            .collect();
        self.draw_arcs(&arcs)
    }

    /// Fill in a polygon defined by the given set of points.
    fn fill_polygon(&mut self, points: &[Point]) -> crate::Result;

    /// Fill in a path.
    #[inline]
    fn fill_path(&mut self, mut path: Path) -> crate::Result {
        // if the path isn't closed, close it
        path.close();

        let points: Vec<Point> = path.into_points().collect();
        self.fill_polygon(&points)
    }

    /// Fill in a rectangle.
    #[inline]
    fn fill_rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        self.fill_polygon(&[
            Point { x: x1, y: y1 },
            Point { x: x2, y: y1 },
            Point { x: x2, y: y2 },
            Point { x: x1, y: y2 },
            Point { x: x1, y: y1 },
        ])
    }

    /// Fill in several rectangles.
    #[inline]
    fn fill_rectangles(&mut self, rects: &[Rect]) -> crate::Result {
        rects
            .iter()
            .copied()
            .try_for_each(|Rectangle { x1, y1, x2, y2 }| self.fill_rectangle(x1, y1, x2, y2))
    }

    /// Fill in an arc.
    #[inline]
    fn fill_arc(
        &mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        start: Angle,
        end: Angle,
    ) -> crate::Result {
        let arc = GeometricArc {
            x1,
            y1,
            x2,
            y2,
            start,
            end,
        };
        let xc = (x1 + x2) / 2;
        let yc = (y1 + y2) / 2;
        let pts: Vec<Point> = iter::once(Point { x: xc, y: yc })
            .chain(arc.into_points())
            .chain(iter::once(Point { x: xc, y: yc }))
            .collect();
        self.fill_polygon(&pts)
    }

    /// Fill in several arcs.
    #[inline]
    fn fill_arcs(&mut self, arcs: &[GeometricArc]) -> crate::Result {
        arcs.iter().copied().try_for_each(
            |GeometricArc {
                 x1,
                 y1,
                 x2,
                 y2,
                 start,
                 end,
             }| self.fill_arc(x1, y1, x2, y2, start, end),
        )
    }

    /// Fill in an ellipse.
    #[inline]
    fn fill_ellipse(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        self.fill_arc(x1, y1, x2, y2, Angle::ZERO, Angle::FULL_CIRCLE)
    }

    /// Fill in several ellipses.
    #[inline]
    fn fill_ellipses(&mut self, rects: &[Rectangle]) -> crate::Result {
        let arcs: Vec<GeometricArc> = rects
            .iter()
            .copied()
            .map(|Rectangle { x1, y1, x2, y2 }| GeometricArc {
                x1,
                y1,
                x2,
                y2,
                start: Angle::ZERO,
                end: Angle::FULL_CIRCLE,
            })
            .collect();
        self.fill_arcs(&arcs)
    }
}
