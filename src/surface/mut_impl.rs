// MIT/Apache2 License

use super::{Surface, SurfaceFeatures};
use crate::{Blur, Color, Context, Ellipse, FillRule, Image, ImageFormat};
use lyon_geom::{Angle, Arc, CubicBezierSegment, LineSegment, Point, Rect};
use lyon_path::{Path, PathBuffer, PathBufferSlice, PathSlice};

impl<S: Surface + ?Sized> Surface for &mut S {
    #[inline]
    fn features(&self) -> SurfaceFeatures {
        (**self).features()
    }
    #[inline]
    fn set_stroke(&mut self, color: Color) -> crate::Result {
        (**self).set_stroke(color)
    }
    #[inline]
    fn set_fill(&mut self, fill: FillRule<'_>) -> crate::Result {
        (**self).set_fill(fill)
    }
    #[inline]
    fn set_line_width(&mut self, lw: u32) -> crate::Result {
        (**self).set_line_width(lw)
    }
    #[inline]
    fn flush(&mut self) -> crate::Result {
        (**self).flush()
    }
    #[inline]
    fn context(&mut self) -> &mut dyn Context {
        (**self).context()
    }
    #[inline]
    fn create_image(
        &mut self,
        image_bytes: &[u8],
        width: u32,
        height: u32,
        format: ImageFormat,
    ) -> crate::Result<Image> {
        (**self).create_image(image_bytes, width, height, format)
    }
    #[inline]
    fn destroy_image(&mut self, image: Image) -> crate::Result {
        (**self).destroy_image(image)
    }
    #[inline]
    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> crate::Result {
        (**self).draw_line(x1, y1, x2, y2)
    }
    #[inline]
    fn draw_lines(&mut self, lines: &[LineSegment<f32>]) -> crate::Result {
        (**self).draw_lines(lines)
    }
    #[inline]
    fn draw_path(&mut self, path: PathSlice<'_>) -> crate::Result {
        (**self).draw_path(path)
    }
    #[inline]
    fn draw_path_owned(&mut self, path: Path) -> crate::Result {
        (**self).draw_path_owned(path)
    }
    #[inline]
    fn draw_path_buffer(&mut self, b: PathBufferSlice<'_>) -> crate::Result {
        (**self).draw_path_buffer(b)
    }
    #[inline]
    fn draw_path_buffer_owned(&mut self, b: PathBuffer) -> crate::Result {
        (**self).draw_path_buffer_owned(b)
    }
    #[inline]
    fn draw_bezier(&mut self, bezier: CubicBezierSegment<f32>) -> crate::Result {
        (**self).draw_bezier(bezier)
    }
    #[inline]
    fn draw_beziers(&mut self, beziers: &[CubicBezierSegment<f32>]) -> crate::Result {
        (**self).draw_beziers(beziers)
    }
    #[inline]
    fn draw_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32) -> crate::Result {
        (**self).draw_rectangle(x, y, width, height)
    }
    #[inline]
    fn draw_rectangles(&mut self, rects: &[Rect<f32>]) -> crate::Result {
        (**self).draw_rectangles(rects)
    }
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
        (**self).draw_arc(xcenter, ycenter, xradius, yradius, start_angle, sweep_angle)
    }
    #[inline]
    fn draw_arcs(&mut self, arcs: &[Arc<f32>]) -> crate::Result {
        (**self).draw_arcs(arcs)
    }
    #[inline]
    fn draw_ellipse(
        &mut self,
        xcenter: f32,
        ycenter: f32,
        xradius: f32,
        yradius: f32,
    ) -> crate::Result {
        (**self).draw_ellipse(xcenter, ycenter, xradius, yradius)
    }
    #[inline]
    fn draw_ellipses(&mut self, ellipses: &[Ellipse]) -> crate::Result {
        (**self).draw_ellipses(ellipses)
    }
    #[inline]
    fn fill_points(&mut self, points: &[Point<f32>]) -> crate::Result {
        (**self).fill_points(points)
    }
    #[inline]
    fn fill_path(&mut self, path: PathSlice<'_>) -> crate::Result {
        (**self).fill_path(path)
    }
    #[inline]
    fn fill_path_owned(&mut self, path: Path) -> crate::Result {
        (**self).fill_path_owned(path)
    }
    #[inline]
    fn fill_path_buffer(&mut self, buffer: PathBufferSlice<'_>) -> crate::Result {
        (**self).fill_path_buffer(buffer)
    }
    #[inline]
    fn fill_path_buffer_owned(&mut self, buffer: PathBuffer) -> crate::Result {
        (**self).fill_path_buffer_owned(buffer)
    }
    #[inline]
    fn fill_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32) -> crate::Result {
        (**self).fill_rectangle(x, y, width, height)
    }
    #[inline]
    fn fill_rectangles(&mut self, rects: &[Rect<f32>]) -> crate::Result {
        (**self).fill_rectangles(rects)
    }
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
        (**self).fill_arc(xcenter, ycenter, xradius, yradius, start_angle, sweep_angle)
    }
    #[inline]
    fn fill_arcs(&mut self, arcs: &[Arc<f32>]) -> crate::Result {
        (**self).fill_arcs(arcs)
    }
    #[inline]
    fn fill_ellipse(
        &mut self,
        xcenter: f32,
        ycenter: f32,
        xradius: f32,
        yradius: f32,
    ) -> crate::Result {
        (**self).fill_ellipse(xcenter, ycenter, xradius, yradius)
    }
    #[inline]
    fn fill_ellipses(&mut self, ellipses: &[Ellipse]) -> crate::Result {
        (**self).fill_ellipses(ellipses)
    }
    #[inline]
    fn copy_image(
        &mut self,
        image: Image,
        src_x: i32,
        src_y: i32,
        dst_x: i32,
        dst_y: i32,
        width: u32,
        height: u32,
    ) -> crate::Result {
        (**self).copy_image(image, src_x, src_y, dst_x, dst_y, width, height)
    }
    #[inline]
    fn set_clipping_mask(&mut self, image: Image) -> crate::Result {
        (**self).set_clipping_mask(image)
    }
    #[inline]
    fn remove_clipping_mask(&mut self) -> crate::Result {
        (**self).remove_clipping_mask()
    }
    #[inline]
    fn set_transform(&mut self, matrix: [f32; 9]) -> crate::Result {
        (**self).set_transform(matrix)
    }
    #[inline]
    fn remove_transform(&mut self) -> crate::Result {
        (**self).remove_transform()
    }
    #[inline]
    fn set_blur(&mut self, blur: Blur) -> crate::Result {
        (**self).set_blur(blur)
    }
    #[inline]
    fn remove_blur(&mut self) -> crate::Result {
        (**self).remove_blur()
    }
}
