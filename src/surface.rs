// MIT/Apache2 License

use crate::{fill::FillRule, path_from_curve, path_to_lines, Color, Ellipse};
use lyon_geom::{Angle, Arc, CubicBezierSegment, LineSegment, Point, Rect};
use lyon_path::{Event as PathEvent, Path, PathBuffer, PathBufferSlice, PathSlice};
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
    /// Does this surface support floats? If not, all numbers will be rounded
    /// down.
    pub floats: bool,
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
    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> crate::Result;
    /// Draw several lines. In many cases this is more efficient than drawing a single line in a loop.
    #[inline]
    fn draw_lines(&mut self, lines: &[LineSegment<f32>]) -> crate::Result {
        lines.iter().copied().try_for_each(
            |LineSegment {
                 from: Point { x: x1, y: y1 },
                 to: Point { x: x2, y: y2 },
             }| self.draw_line(x1, y1, x2, y2),
        )
    }

    /// Draw a path.
    #[inline]
    fn draw_path(&mut self, path: PathSlice<'_>) -> crate::Result {
        let lines: Vec<LineSegment> = path_to_lines(path.iter()).collect();
        self.draw_lines(&lines)
    }
    /// Draw an owned path.
    #[inline]
    fn draw_path_owned(&mut self, path: Path) -> crate::Result {
        let lines: Vec<LineSegment> = path_to_lines(path.iter()).collect();
        self.draw_lines(&lines)
    }
    /// Draw several paths.
    #[inline]
    fn draw_paths(&mut self, paths: PathBufferSlice<'_>) -> crate::Result {
        let lines: Vec<LineSegment> = paths
            .indices()
            .flat_map(|index| path_to_lines(paths.get(index).iter()))
            .collect();
        self.draw_lines(&lines)
    }
    /// Draw several paths, if we own the paths.
    #[inline]
    fn draw_paths_owned(&mut self, paths: PathBuffer) -> crate::Result {
        let lines: Vec<LineSegment> = paths
            .indices()
            .flat_map(|path| path_to_lines(paths.get(index).iter()))
            .collect();
        self.draw_lines(&lines)
    }

    /// Draw a bezier curve.
    #[inline]
    fn draw_bezier_curve(&mut self, curve: CubicBezierSegment<f32>) -> crate::Result {
        // Although we could implement this in terms of draw_lines(), most Surfaces that are actually used have
        // optimizations to make drawing paths faster. For the ones that don't, the allocation incurred here is
        // likely not much of a big deal since they use IPC, and the overhead here isn't comparable to the
        // overhead of that.
        let path = path_from_curve(curve);
        self.draw_path_owned(path)
    }
    /// Draw several bezier curves. In many cases this is more efficient than drawing a single curve in a loop.
    #[inline]
    fn draw_bezier_curves(&mut self, curves: &[BezierCurve]) -> crate::Result {
        let paths: PathBuffer = curves
            .iter()
            .copied()
            .map(|curve| path_from_curve(curve))
            .collect();
        self.draw_paths_owned(paths)
    }

    /// Draw a rectangle.
    #[inline]
    fn draw_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32) -> crate::Result {
        let mut builder = Path::builder();
        builder.begin(Point { x, y });
        builder.line_to(Point { x: x + width, y });
        builder.line_to(Point {
            x: x + width,
            y: y + height,
        });
        builder.line_to(Point { x, y: y + height });
        builder.close();

        self.draw_path_owned(builder.build())
    }

    /// Draw several rectangles. In many cases this is more efficient than drawing a single rectangle in a loop.
    #[inline]
    fn draw_rectangles(&mut self, rects: &[Rectangle]) -> crate::Result {
        let paths: PathBuffer = rects
            .iter()
            .copied()
            .map(
                |Rectangle {
                     origin: Point { x, y },
                     size: Size { width, height },
                 }| {
                    let mut builder = Path::builder();
                    builder.begin(Point { x, y });
                    builder.line_to(Point { x: x + width, y });
                    builder.line_to(Point {
                        x: x + width,
                        y: y + height,
                    });
                    builder.line_to(Point { x, y: y + height });
                    builder.close();

                    builder.build()
                },
            )
            .collect();

        self.draw_paths_owned(paths)
    }

    /// Draw an arc.
    #[inline]
    fn draw_arc(
        &mut self,
        xcenter: f32,
        ycenter: f32,
        xradius: f32,
        yradius: f32,
        start_angle: Angle<f32>,
        sweep_angle: Angle<f32>,
    ) -> crate::Result {
        match path_from_arc(Arc {
            center: Point {
                x: xcenter,
                y: ycenter,
            },
            radii: Vector {
                x: xradius,
                y: yradius,
            },
            start_angle,
            sweep_angle,
            x_rotation: Angle { radians: 0.0 },
        }) {
            Some(arc) => self.draw_path_owned(arc),
            None => Ok(()),
        }
    }

    /// Draw several arcs.
    #[inline]
    fn draw_arcs(&mut self, arcs: &[Arc<f32>]) -> crate::Result {
        self.draw_paths_owned(
            arcs.iter()
                .copied()
                .filter_map(|arc| path_from_arc(arc))
                .collect(),
        )
    }

    /// Draw an ellipse.
    #[inline]
    fn draw_ellipse(
        &mut self,
        xcenter: f32,
        ycenter: f32,
        xradius: f32,
        yradius: f32,
    ) -> crate::Result {
        self.draw_arc(
            xradius,
            yradius,
            xradius,
            yradius,
            Angle { radians: 0.0 },
            Angle {
                radians: f32::consts::PI * 2.0,
            },
        )
    }

    /// Draw several ellipses.
    #[inline]
    fn draw_ellipses(&mut self, rects: &[Ellipse]) -> crate::Result {
        let arcs: Vec<Arc<f32>> = rects
            .iter()
            .copied()
            .map(|Ellipse { center, radii }| Arc {
                center,
                radii,
                start_angle: Angle { radians: 0.0 },
                sweep_angle: Angle {
                    radians: 2 * f32::consts::PI,
                },
                x_rotation: Angle { radians: 0.0 },
            })
            .collect();
        self.draw_arcs(&arcs)
    }

    /// Fill in a polygon defined by the given set of points.
    fn fill_polygon(&mut self, points: &[Point<f32>]) -> crate::Result;

    /// Fill in an owned path.
    #[inline]
    fn fill_path_owned(&mut self, path: Path) -> crate::Result {
        let points: Vec<Point<f32>> = path_to_points(path.iter()).collect();
        self.fill_polygon(&points)
    }

    /// Fill in a path slice.
    #[inline]
    fn fill_path(&mut self, path: PathSlice<'_>) -> crate::Result {
        let points: Vec<Point<f32>> = path_to_points(path.iter()).collect();
        self.fill_polygon(&points)
    }

    /// Fill in a series of paths in a path buffer.
    #[inline]
    fn fill_paths_owned(&mut self, paths: PathBuffer) -> crate::Result {
        paths
            .indices()
            .try_for_each(|index| self.fill_path(paths.get(index)))
    }

    /// Fill in a series of paths in a path buffer.
    #[inline]
    fn fill_paths(&mut self, paths: PathBufferSlice<'_>) -> crate::Result {
        paths
            .indices()
            .try_for_each(|index| self.fill_path(paths.get(index)))
    }

    /// Fill in a rectangle.
    #[inline]
    fn fill_rectangle(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> crate::Result {
        self.fill_polygon(&[
            Point { x: x1, y: y1 },
            Point { x: x2, y: y1 },
            Point { x: x2, y: y2 },
            Point { x: x1, y: y2 },
        ])
    }

    /// Fill in several rectangles.
    #[inline]
    fn fill_rectangles(&mut self, rects: &[Rect<f32>]) -> crate::Result {
        rects
            .iter()
            .copied()
            .try_for_each(|Rectangle { x1, y1, x2, y2 }| self.fill_rectangle(x1, y1, x2, y2))
    }

    /// Fill in an arc.
    #[inline]
    fn fill_arc(
        &mut self,
        xcenter: f32,
        ycenter: f32,
        xradius: f32,
        yradius: f32,
        start_angle: Angle<f32>,
        sweep_angle: Angle<f32>,
    ) -> crate::Result {
        let arc = Arc {
            center: Point {
                x: xcenter,
                y: ycenter,
            },
            radii: Vector {
                x: xradius,
                y: yradius,
            },
            start_angle,
            sweep_angle,
            x_rotation: Angle { radians: 0.0 },
        };
        match path_from_arc_closed(arc) {
            Some(path) => self.fill_path(path),
            None => Ok(()),
        }
    }

    /// Fill in several arcs.
    #[inline]
    fn fill_arcs(&mut self, arcs: &[GeometricArc]) -> crate::Result {
        let paths: PathBuffer = arcs
            .iter()
            .copied()
            .filter_map(|arc| path_from_arc_closed(arc))
            .collect();
        self.fill_paths_owned(paths)
    }

    /// Fill in an ellipse.
    #[inline]
    fn fill_ellipse(
        &mut self,
        xcenter: f32,
        ycenter: f32,
        xradius: f32,
        yradius: f32,
    ) -> crate::Result {
        self.fill_arc(
            xcenter,
            ycenter,
            xradius,
            yradius,
            Angle { radians: 0 },
            Angle {
                radians: 2.0 * f32::consts::PI,
            },
        )
    }

    /// Fill in several ellipses.
    #[inline]
    fn fill_ellipses(&mut self, rects: &[Ellipse]) -> crate::Result {
        let arcs: Vec<Arc<f32>> = rects
            .iter()
            .copied()
            .map(|Ellipse { center, radii }| GeometricArc {
                center,
                radii,
                start_angle: Angle { radians: 0.0 },
                sweep_angle: Angle {
                    radians: 2.0 * f32::consts::PI,
                },
                x_rotation: Angle { radians: 0.0 },
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
