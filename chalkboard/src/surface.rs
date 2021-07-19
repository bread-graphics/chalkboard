// MIT/Apache2 License

use crate::{
    color::Color,
    fill::FillRule,
    geometry::{Angle, BezierCurve, GeometricArc, Line, Point, Rectangle},
    path::{Path, PathSegment, PathSegmentType, PathSlice},
};
use std::{array::IntoIter as ArrayIter, iter};

#[cfg(feature = "async")]
use crate::util::GenericResult;
#[cfg(feature = "async")]
use futures_lite::{
    future::FutureExt,
    stream::{self, StreamExt},
};

/// Features that a surface can support.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SurfaceFeatures {
    /// Does this surface support drawing gradients?
    pub gradients: bool,
}

/// A surface which drawing commands can be applied to.
pub trait Surface {
    /// The set of features this surface supports.
    fn features(&self) -> SurfaceFeatures;
    /// Set the color used to draw lines.
    fn set_stroke(&mut self, color: Color) -> crate::Result;
    /// Set the rule used to fill shapes.
    fn set_fill(&mut self, rule: FillRule) -> crate::Result;
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
    fn draw_path(&mut self, path: &PathSlice) -> crate::Result {
        let lines: Vec<Line> = path.iter_lines().collect();
        self.draw_lines(&lines)
    }
    /// Draw an owned path.
    #[inline]
    fn draw_path_owned(&mut self, path: Path) -> crate::Result {
        let lines: Vec<Line> = path.into_iter_lines().collect();
        self.draw_lines(&lines)
    }
    /// Draw several paths.
    #[inline]
    fn draw_paths(&mut self, paths: &[Path]) -> crate::Result {
        let lines: Vec<Line> = paths.iter().flat_map(|path| path.iter_lines()).collect();
        self.draw_lines(&lines)
    }
    /// Draw several paths, if we own the paths.
    #[inline]
    fn draw_paths_owned(&mut self, paths: Vec<Path>) -> crate::Result {
        let lines: Vec<Line> = paths
            .into_iter()
            .flat_map(|path| path.into_iter_lines())
            .collect();
        self.draw_lines(&lines)
    }

    /// Draw a bezier curve.
    #[inline]
    fn draw_bezier_curve(&mut self, curve: BezierCurve) -> crate::Result {
        // Although we could implement this in terms of draw_lines(), most Surfaces that are actually used have
        // optimizations to make drawing paths faster. This is a gamble.
        let path = Path::from(curve);
        self.draw_path_owned(path)
    }
    /// Draw several bezier curves. In many cases this is more efficient than drawing a single curve in a loop.
    #[inline]
    fn draw_bezier_curves(&mut self, curves: &[BezierCurve]) -> crate::Result {
        let paths: Vec<Path> = curves.iter().copied().map(|curve| curve.into()).collect();
        self.draw_paths_owned(paths)
    }

    /// Draw a rectangle.
    #[inline]
    fn draw_rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        let path = Path::polyline([
            Point { x: x1, y: y1 },
            Point { x: x2, y: y1 },
            Point { x: x2, y: y2 },
            Point { x: x2, y: y1 },
            Point { x: x1, y: y1 },
        ]);

        self.draw_path_owned(path)
    }

    /// Draw several rectangles. In many cases this is more efficient than drawing a single rectangle in a loop.
    #[inline]
    fn draw_rectangles(&mut self, rects: &[Rectangle]) -> crate::Result {
        let paths: Vec<Path> = rects
            .iter()
            .copied()
            .map(|Rectangle { x1, y1, x2, y2 }| {
                Path::polyline([
                    Point { x: x1, y: y1 },
                    Point { x: x2, y: y1 },
                    Point { x: x2, y: y2 },
                    Point { x: x2, y: y1 },
                    Point { x: x1, y: y1 },
                ])
            })
            .collect();

        self.draw_paths_owned(paths)
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
        .into_lines()
        .collect();
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
        self.draw_arc(x1, y1, x2, y2, Angle::ZERO, Angle::FULL_CIRCLE)
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
    fn fill_rectangles(&mut self, rects: &[Rectangle]) -> crate::Result {
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

/// A surface which drawing commands can be applied to, in a non-blocking way.
#[cfg(feature = "async")]
pub trait AsyncSurface: Send {
    /// The set of features this surface supports.
    fn features(&self) -> SurfaceFeatures;
    /// Set the color used to draw lines.
    fn set_stroke_color_async<'future>(&'future mut self, color: Color) -> GenericResult<'future>;
    /// Set the color used to fill shapes.
    fn set_fill_color_async<'future>(&'future mut self, color: Color) -> GenericResult<'future>;
    /// Set the width used to draw lines.
    fn set_line_width_async<'future>(&'future mut self, width: usize) -> GenericResult<'future>;

    /// Flush all commands passed to this surface to its target.
    fn flush_async<'future>(&'future mut self) -> GenericResult<'future>;

    /// Draw a single line.
    fn draw_line_async<'future>(
        &'future mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    ) -> GenericResult<'future>;
    /// Draw several lines. In many cases this is more efficient than drawing a single line in a loop.
    #[inline]
    fn draw_lines_async<'future, 'a, 'b>(&'a mut self, lines: &'b [Line]) -> GenericResult<'future>
    where
        'a: 'future,
        'b: 'future,
    {
        Box::pin(async move {
            for line in lines {
                self.draw_line_async(line.x1, line.y1, line.x2, line.y2)
                    .await?;
            }
            Ok(())
        })
    }

    /// Draw a path.
    #[inline]
    fn draw_path_async<'future>(&'future mut self, path: Path) -> GenericResult<'future> {
        let lines = path.into_lines();
        Box::pin(async move { self.draw_lines_async(&lines).await })
    }

    /// Draw a bezier curve.
    #[inline]
    fn draw_bezier_curve_async<'future>(
        &'future mut self,
        curve: BezierCurve,
    ) -> GenericResult<'future> {
        let lines: Vec<Line> = curve.into_lines().collect();
        Box::pin(async move { self.draw_lines_async(&lines).await })
    }

    /// Draw several bezier curves. In many cases this is more efficient than drawing a single curve in a loop.
    #[inline]
    fn draw_bezier_curves_async<'future>(
        &'future mut self,
        curves: &[BezierCurve],
    ) -> GenericResult<'future> {
        let lines: Vec<Line> = curves
            .iter()
            .copied()
            .flat_map(|curve| curve.into_lines())
            .collect();
        Box::pin(async move { self.draw_lines_async(&lines).await })
    }

    /// Draw a rectangle.
    #[inline]
    fn draw_rectangle_async<'future>(
        &'future mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    ) -> GenericResult<'future> {
        let lines: [Line; 4] = [
            Line { x1, y1, x2, y2: y1 },
            Line { x1: x2, y1, x2, y2 },
            Line { x1, y1: y2, x2, y2 },
            Line { x1, y1, x2: x1, y2 },
        ];

        Box::pin(async move { self.draw_lines_async(&lines).await })
    }

    /// Draw several rectangles. In many cases this is more efficient than drawing a single rectangle in a loop.
    #[inline]
    fn draw_rectangles_async<'future, 'a, 'b>(
        &'a mut self,
        rects: &'b [Rectangle],
    ) -> GenericResult<'future>
    where
        'a: 'future,
        'b: 'future,
    {
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
        Box::pin(async move { self.draw_lines_async(&lines).await })
    }

    /// Draw an arc.
    #[inline]
    fn draw_arc_async<'future>(
        &'future mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        start: Angle,
        end: Angle,
    ) -> GenericResult<'future> {
        let lines: Vec<Line> = GeometricArc {
            x1,
            y1,
            x2,
            y2,
            start,
            end,
        }
        .into_lines()
        .collect();
        Box::pin(async move { self.draw_lines_async(&lines).await })
    }

    /// Draw several arcs.
    #[inline]
    fn draw_arcs_async<'future, 'a, 'b>(
        &'a mut self,
        arcs: &'b [GeometricArc],
    ) -> GenericResult<'future>
    where
        'a: 'future,
        'b: 'future,
    {
        let lines: Vec<Line> = arcs
            .iter()
            .copied()
            .flat_map(|arc| arc.into_lines())
            .collect();
        Box::pin(async move { self.draw_lines_async(&lines).await })
    }

    /// Draw an ellipse.
    #[inline]
    fn draw_ellipse_async<'future>(
        &'future mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    ) -> GenericResult<'future> {
        self.draw_arc_async(x1, y1, x2, y1, Angle::ZERO, Angle::FULL_CIRCLE)
    }

    /// Draw several ellipses.
    #[inline]
    fn draw_ellipses_async<'future, 'a, 'b>(
        &'a mut self,
        rects: &'b [Rectangle],
    ) -> GenericResult<'future>
    where
        'a: 'future,
        'b: 'future,
    {
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
        Box::pin(async move { self.draw_arcs_async(&arcs).await })
    }

    /// Fill in a polygon defined by the given set of points.
    fn fill_polygon_async<'future, 'a, 'b>(
        &'a mut self,
        points: &'b [Point],
    ) -> GenericResult<'future>
    where
        'a: 'future,
        'b: 'future;

    /// Fill in a path.
    #[inline]
    fn fill_path_async<'future>(&'future mut self, mut path: Path) -> GenericResult<'future> {
        // if the path isn't closed, close it
        path.close();

        let points: Vec<Point> = path.into_points().collect();
        Box::pin(async move { self.fill_polygon_async(&points).await })
    }

    /// Fill in a rectangle.
    #[inline]
    fn fill_rectangle_async<'future>(
        &'future mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    ) -> GenericResult<'future> {
        Box::pin(async move {
            self.fill_polygon_async(&[
                Point { x: x1, y: y1 },
                Point { x: x2, y: y1 },
                Point { x: x2, y: y2 },
                Point { x: x1, y: y2 },
                Point { x: x1, y: y1 },
            ])
            .await
        })
    }

    /// Fill in several rectangles.
    #[inline]
    fn fill_rectangles_async<'future, 'a, 'b>(
        &'a mut self,
        rects: &'b [Rectangle],
    ) -> GenericResult<'future>
    where
        'a: 'future,
        'b: 'future,
    {
        Box::pin(async move {
            for rect in rects {
                self.fill_rectangle_async(rect.x1, rect.y1, rect.x2, rect.y2)
                    .await?;
            }
            Ok(())
        })
    }

    /// Fill in an arc.
    #[inline]
    fn fill_arc_async<'future>(
        &'future mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        start: Angle,
        end: Angle,
    ) -> GenericResult<'future> {
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
        Box::pin(async move { self.fill_polygon_async(&pts).await })
    }

    /// Fill in several arcs.
    #[inline]
    fn fill_arcs_async<'future, 'a, 'b>(
        &'a mut self,
        arcs: &'b [GeometricArc],
    ) -> GenericResult<'future>
    where
        'a: 'future,
        'b: 'future,
    {
        Box::pin(async move {
            for arc in arcs {
                self.fill_arc_async(arc.x1, arc.y1, arc.x2, arc.y2, arc.start, arc.end)
                    .await?;
            }
            Ok(())
        })
    }

    /// Fill in an ellipse.
    #[inline]
    fn fill_ellipse_async<'future>(
        &'future mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    ) -> GenericResult<'future> {
        self.fill_arc_async(x1, y1, x2, y2, Angle::ZERO, Angle::FULL_CIRCLE)
    }

    /// Fill in several ellipses.
    #[inline]
    fn fill_ellipses_async<'future, 'a, 'b>(
        &'a mut self,
        rects: &'b [Rectangle],
    ) -> GenericResult<'future>
    where
        'a: 'future,
        'b: 'future,
    {
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
        Box::pin(async move { self.fill_arcs_async(&arcs).await })
    }
}
