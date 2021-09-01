// MIT/Apache2 License

use super::{Blur, Color, Context, Ellipse, FillRule, Image, ImageFormat};
use lyon_geom::{Angle, Arc, CubicBezierSegment, LineSegment, Point, Rect};
use lyon_path::{Path, PathBuffer, PathBufferSlice, PathSlice};

/// Default implementations of `Surface` functions.
mod defaults;
/// Provides the `SurfaceFeatures` type.
mod features;
/// Implements `Surface` on `&mut Surface`.
mod mut_impl;

pub use features::SurfaceFeatures;

/// Something that can be drawn upon; otherwise known as the whole point of this crate.
///
/// `Surface`s are usually windows, images, or other buffers containing pixels that can be modified though
/// system drawing APIs. The `Surface` trait provides a common API that allows one to abstract over the given
/// system drawing API.
///
/// No matter what, `Surface`s should be capable of the following:
///
/// * Returning a list of "features" that the `Surface` implements by returning a [`SurfaceFeatures`] object
///   from the `features()` method.
/// * Setting the current stroke type and fill type using the `set_stroke` and `set_fill` methods. `set_fill`
///   is only required to process `FillRule::Solid` and can return `NotSupported` if other fill types are
///   used.
/// * Drawing the outlines of geometric primitives. Implementors only need to implement `draw_line()`; the
///   remaining methods are implemented in terms of drawing lines. However, it is usually more efficient
///   to reimplement the rest of the `draw_*` methods using the system drawing API as well.
/// * Filling geometric primitives. Implementors only need to implement `fill_points()`; however, in a similar
///   vein to above, implementing the rest of the `fill_*` methods tends to be more efficient.
/// * Providing a reference to the [`Context`] backing this `Surface`. This is used for the default
///   implementation of `create_image` and `destroy_image`.
/// * Copying the data in an [`Image`] onto this `Surface` at a specified point.
/// * Setting an `Image` to a "clipping region" where drawing will be clipped to.
///
/// If the `SurfaceFeatures` says that this surface implements additional features, it is also capable of the
/// following. Note that it is expected that incapable surfaces should return `NotSupported` if it attempts to
/// do these tasks:
///
/// * If the `gradients` field is enabled, the `set_fill` method is expected to be able to handle non-solid
///   fills, such as linear, radial and conical gradients.
/// * If the `transparancy` field is enabled, `Image`s can use `ImageFormat`s that contain alpha channels.
///   Normally, the alpha channel is ignored.
/// * If the `floats` field is enabled, floating point values passed into the drawing functions will be
///   processed correctly. Normal behavior is for floats to be rounded down to integers.
/// * If the `transforms` field is enabled, `set_transform` and `remove_transform` can be used to apply
///   matrix transformations to drawing operations.
/// * If the `blurs` field is enabled, `set_blur` and `remove_blur` can be used to apply blur effects to
///   drawing.
pub trait Surface {
    /* Setup */

    /// Get an enumeration of the features that this `Surface` is capable of.
    ///
    /// See the [`SurfaceFeatures`]
    /// structure for more information.
    fn features(&self) -> SurfaceFeatures;

    /// Set the solid color used to draw outlines.
    fn set_stroke(&mut self, color: Color) -> crate::Result;
    /// Set the filling rule used to fill in shapes.
    fn set_fill(&mut self, fill: FillRule<'_>) -> crate::Result;
    /// Set the width of the lines that the outlines are drawn in.
    fn set_line_width(&mut self, line_width: u32) -> crate::Result;

    /// Flush all drawing operations down the connection, if necessary.
    fn flush(&mut self) -> crate::Result {
        Ok(())
    }

    /// Get the underlying `Context` backing this `Surface`.
    fn context(&mut self) -> &mut dyn Context;
    /// Create a new `Image`.
    #[inline]
    fn create_image(
        &mut self,
        image_bytes: &[u8],
        width: u32,
        height: u32,
        format: ImageFormat,
    ) -> crate::Result<Image> {
        self.context()
            .create_image(image_bytes, width, height, format)
    }
    /// Destroy an existing `Image`.
    #[inline]
    fn destroy_image(&mut self, image: Image) -> crate::Result {
        self.context().destroy_image(image)
    }

    /* Lines and Outlines of Geometric Primitives */

    /// Draw a straight line segment from (x1, y1) to (x2, y2).
    ///
    /// This uses the stroke properties to do drawing.
    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> crate::Result;
    /// Draw several line segments.
    ///
    /// This is preferred to calling `draw_line()` in a loop if you have several
    /// line segments, since it can sometimes batch process several lines into one request.
    #[inline]
    fn draw_lines(&mut self, lines: &[LineSegment<f32>]) -> crate::Result {
        defaults::draw_lines(self, lines)
    }
    /// Draw a path slice using the current stroke options.
    #[inline]
    fn draw_path(&mut self, path: PathSlice<'_>) -> crate::Result {
        defaults::draw_path(self, path)
    }
    /// Draw an owned path using the current stroke options.
    #[inline]
    fn draw_path_owned(&mut self, path: Path) -> crate::Result {
        self.draw_path(path.as_slice())
    }
    /// Draw a series of paths using the current stroke options.
    #[inline]
    fn draw_path_buffer(&mut self, buffer: PathBufferSlice<'_>) -> crate::Result {
        defaults::draw_path_buffer(self, buffer)
    }
    /// Draw a series of owned paths using the current stroke options.
    #[inline]
    fn draw_path_buffer_owned(&mut self, buffer: PathBuffer) -> crate::Result {
        self.draw_path_buffer(buffer.as_slice())
    }
    /// Draw a cubic bezier curve segment using the current stroke options.
    #[inline]
    fn draw_bezier(&mut self, bezier: CubicBezierSegment<f32>) -> crate::Result {
        defaults::draw_bezier(self, bezier)
    }
    /// Draw several cubic bezier curve segments.
    #[inline]
    fn draw_beziers(&mut self, beziers: &[CubicBezierSegment<f32>]) -> crate::Result {
        defaults::draw_beziers(self, beziers)
    }

    /// Draw the outline of a rectangle using the current stroke options.
    #[inline]
    fn draw_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32) -> crate::Result {
        defaults::draw_rectangle(self, x, y, width, height)
    }
    /// Draw the outline of several rectangles.
    ///
    /// This is preferred to calling `draw_rectangle()` in a loop.
    #[inline]
    fn draw_rectangles(&mut self, rects: &[Rect<f32>]) -> crate::Result {
        defaults::draw_rectangles(self, rects)
    }
    /// Draw the outline of an arc using the current stroke options.
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
        defaults::draw_arc(
            self,
            xcenter,
            ycenter,
            xradius,
            yradius,
            start_angle,
            sweep_angle,
        )
    }
    /// Draw the outline of several arcs.
    ///
    /// This is preferred to calling `draw_arc()` in a loop. Note that the
    /// `x_rotation` field is completely ignored.
    #[inline]
    fn draw_arcs(&mut self, arcs: &[Arc<f32>]) -> crate::Result {
        defaults::draw_arcs(self, arcs)
    }
    /// Draw the outline of an ellipse using the current stroke options.
    #[inline]
    fn draw_ellipse(
        &mut self,
        xcenter: f32,
        ycenter: f32,
        xradius: f32,
        yradius: f32,
    ) -> crate::Result {
        self.draw_arc(
            xcenter,
            ycenter,
            xradius,
            yradius,
            Angle { radians: 0.0 },
            Angle {
                radians: std::f32::consts::PI * 2.0,
            },
        )
    }
    /// Draw the outlines of several ellipses.
    #[inline]
    fn draw_ellipses(&mut self, ellipses: &[Ellipse]) -> crate::Result {
        defaults::draw_ellipses(self, ellipses)
    }

    /* Fills and Geometric Primitives */

    /// Fill in a polygon defined by a series of points representing its outline.
    fn fill_points(&mut self, points: &[Point<f32>]) -> crate::Result;
    /// Fill in a polygon defined by a path. The path is automatically closed if it hasn't been already.
    #[inline]
    fn fill_path(&mut self, path: PathSlice<'_>) -> crate::Result {
        defaults::fill_path(self, path)
    }
    /// Fill in a polygon defined by an owned path.
    #[inline]
    fn fill_path_owned(&mut self, path: Path) -> crate::Result {
        self.fill_path(path.as_slice())
    }
    /// Fill in several polygons defined by paths.
    #[inline]
    fn fill_path_buffer(&mut self, buffer: PathBufferSlice<'_>) -> crate::Result {
        defaults::fill_path_buffer(self, buffer)
    }
    /// Fill in several polygons defined by owned paths.
    #[inline]
    fn fill_path_buffer_owned(&mut self, buffer: PathBuffer) -> crate::Result {
        self.fill_path_buffer(buffer.as_slice())
    }

    /// Fill in a rectangle using the current fill rules.
    #[inline]
    fn fill_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32) -> crate::Result {
        defaults::fill_rectangle(self, x, y, width, height)
    }
    /// Fill in several rectangles using the current fill rules.
    #[inline]
    fn fill_rectangles(&mut self, rects: &[Rect<f32>]) -> crate::Result {
        defaults::fill_rectangles(self, rects)
    }
    /// Fill in an arc using the current fill rule.
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
        defaults::fill_arc(
            self,
            xcenter,
            ycenter,
            xradius,
            yradius,
            start_angle,
            sweep_angle,
        )
    }
    /// Fill in several arcs using the current fill rule.
    #[inline]
    fn fill_arcs(&mut self, arcs: &[Arc<f32>]) -> crate::Result {
        defaults::fill_arcs(self, arcs)
    }
    /// Fill in an ellipse using the current fill rule.
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
            Angle { radians: 0.0 },
            Angle {
                radians: std::f32::consts::PI * 2.0,
            },
        )
    }
    /// Fill in several ellpses using the current fill rule.
    #[inline]
    fn fill_ellipses(&mut self, ellipses: &[Ellipse]) -> crate::Result {
        defaults::fill_ellipses(self, ellipses)
    }

    /* Image-Related Functions */

    /// Copy an image onto this surface.
    ///
    /// An area of `(width, height)` is copied from the image, beginning at
    /// `(src_x, src_y)`, to the surface, starting at `(dst_x, dst_y)`. If the `transparancy` surface feature
    /// is enabled, alpha blending is used during copying.
    fn copy_image(
        &mut self,
        image: Image,
        src_x: i32,
        src_y: i32,
        dst_x: i32,
        dst_y: i32,
        width: u32,
        height: u32,
    ) -> crate::Result;

    /// Set a "clipping mask" for this surface.
    ///
    /// The input is expected to be an image; the color values of the
    /// image are used to determine where on the surface can be drawn. Pixels of 0 indicate an area where drawing
    /// may not occur.
    fn set_clipping_mask(&mut self, image: Image) -> crate::Result;
    /// Remove a clipping mask set previously by `set_clipping_mask()`.
    ///
    /// This should be a no-op if `set_clipping_mask()` was not previously called.
    fn remove_clipping_mask(&mut self) -> crate::Result;

    /* Transform Functions */

    /// Sets a matrix to be used as a transformation matrix for this surface.
    ///
    /// The "matrix" is formatted as an
    /// array of floats representing a 3x3 matrix. The matrix is expected to be formatted in column order, where
    /// `matrix[0]` is *a<sub>00</sub>*, `matrix[1]` is *a<sub>10</sub>*, `matrix[3]` is
    /// *a<sub>01</sub>*, and so on and so forth. For users of the [`nalgebra`] crate, this means the result of
    /// `Matrix::as_slice()` can be passed into this after it is converted into an array.
    ///
    /// [`nalgebra`]: https://crates.io/crates/nalgebra
    #[inline]
    fn set_transform(&mut self, matrix: [f32; 9]) -> crate::Result {
        let _ = matrix;
        Err(crate::Error::NotSupported(
            crate::NotSupportedOp::Transforms,
        ))
    }
    /// Removes a transform previously set via `set_transform()`.
    ///
    /// It is reasonable to expect that calling this function will lead to a flush operation occurring.
    #[inline]
    fn remove_transform(&mut self) -> crate::Result {
        Err(crate::Error::NotSupported(
            crate::NotSupportedOp::Transforms,
        ))
    }

    /* Blur Functions */

    /// Apply a blur to this `Surface`.
    ///
    /// All drawing operations from this point until either `set_blur()` is
    /// called again or `remove_blur()` is called will be filtered through this blurring. See the [`Blur`]
    /// enum for more information on how blurring can occur.
    #[inline]
    fn set_blur(&mut self, blur: Blur) -> crate::Result {
        let _ = blur;
        Err(crate::Error::NotSupported(crate::NotSupportedOp::Blurs))
    }
    /// Removes a blurring previously set via `set_blur()`.
    ///
    /// It is reasonable to expect that calling this function will lead to a flush.
    #[inline]
    fn remove_blur(&mut self) -> crate::Result {
        Err(crate::Error::NotSupported(crate::NotSupportedOp::Blurs))
    }
}
