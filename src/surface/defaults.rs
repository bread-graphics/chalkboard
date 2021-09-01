// MIT/Apache2 License

use super::Surface;
use crate::{path_utils, Ellipse};
use lyon_geom::{Angle, Arc, CubicBezierSegment, LineSegment, Point, Rect, Size, Vector};
use lyon_path::{
    builder::PathBuilder, path::Builder, PathBuffer, PathBufferSlice,
    PathEvent, PathSlice,
};

#[inline]
pub(crate) fn draw_lines<S: Surface + ?Sized>(
    s: &mut S,
    lines: &[LineSegment<f32>],
) -> crate::Result {
    lines.iter().copied().try_for_each(
        |LineSegment {
             to: Point { x: x1, y: y1, .. },
             from: Point { x: x2, y: y2, .. },
         }| s.draw_line(x1, y1, x2, y2),
    )
}

#[inline]
pub(crate) fn draw_path<S: Surface + ?Sized>(s: &mut S, path: PathSlice<'_>) -> crate::Result {
    draw_path_events(s, path.iter())
}

#[inline]
pub(crate) fn draw_path_buffer<S: Surface + ?Sized>(
    s: &mut S,
    buffer: PathBufferSlice<'_>,
) -> crate::Result {
    draw_path_events(s, buffer.iter().flat_map(|path| path.iter()))
}

#[inline]
pub(crate) fn draw_bezier<S: Surface + ?Sized>(
    s: &mut S,
    bezier: CubicBezierSegment<f32>,
) -> crate::Result {
    draw_path_events(s, bezier_events(bezier))
}

#[inline]
pub(crate) fn draw_beziers<S: Surface + ?Sized>(
    s: &mut S,
    beziers: &[CubicBezierSegment<f32>],
) -> crate::Result {
    draw_path_events(
        s,
        beziers
            .iter()
            .copied()
            .flat_map(|bezier| bezier_events(bezier)),
    )
}

#[inline]
pub(crate) fn draw_rectangle<S: Surface + ?Sized>(
    s: &mut S,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> crate::Result {
    // many surfaces are optimized to tesselate paths faster than lines, since paths are guaranteed to be
    // connected. this is something of a gamble; we should go back and verify if it's faster later
    let mut builder = Builder::with_capacity(4, 4);
    build_rectangle(&mut builder, x, y, width, height);
    let path = builder.build();

    s.draw_path_owned(path)
}

#[inline]
pub(crate) fn draw_rectangles<S: Surface + ?Sized>(
    s: &mut S,
    rects: &[Rect<f32>],
) -> crate::Result {
    let mut buffer = PathBuffer::with_capacity(rects.len() * 4, rects.len() * 4, rects.len());
    rects.iter().copied().for_each(
        |Rect {
             origin: Point { x, y, .. },
             size: Size { width, height, .. },
         }| {
            let mut builder = buffer.builder();
            build_rectangle(&mut builder, x, y, width, height);
            builder.build();
        },
    );

    s.draw_path_buffer_owned(buffer)
}

#[inline]
pub(crate) fn draw_arc<S: Surface + ?Sized>(
    s: &mut S,
    xcenter: f32,
    ycenter: f32,
    xradius: f32,
    yradius: f32,
    start_angle: Angle<f32>,
    sweep_angle: Angle<f32>,
) -> crate::Result {
    let approx_perimeter_times_3 = (((xcenter + ycenter) * std::f32::consts::PI) as usize) << 2;

    let mut builder = Builder::with_capacity(approx_perimeter_times_3, approx_perimeter_times_3);
    build_arc(
        &mut builder,
        xcenter,
        ycenter,
        xradius,
        yradius,
        start_angle,
        sweep_angle,
    );
    let path = builder.build();

    s.draw_path_owned(path)
}

#[inline]
pub(crate) fn draw_arcs<S: Surface + ?Sized>(s: &mut S, arcs: &[Arc<f32>]) -> crate::Result {
    let mut buffer = PathBuffer::with_capacity(arcs.len(), arcs.len(), arcs.len());
    arcs.iter().copied().for_each(
        |Arc {
             center:
                 Point {
                     x: xcenter,
                     y: ycenter,
                     ..
                 },
             radii:
                 Vector {
                     x: xradius,
                     y: yradius,
                     ..
                 },
             start_angle,
             sweep_angle,
             ..
         }| {
            let mut builder = buffer.builder();
            build_arc(
                &mut builder,
                xcenter,
                ycenter,
                xradius,
                yradius,
                start_angle,
                sweep_angle,
            );
            builder.build();
        },
    );

    s.draw_path_buffer_owned(buffer)
}

#[inline]
pub(crate) fn draw_ellipses<S: Surface + ?Sized>(s: &mut S, ellipses: &[Ellipse]) -> crate::Result {
    let arcs: Vec<Arc<f32>> = ellipses
        .iter()
        .copied()
        .map(|Ellipse { center, radii }| Arc {
            center,
            radii,
            start_angle: Angle { radians: 0.0 },
            sweep_angle: Angle {
                radians: std::f32::consts::PI * 2.0,
            },
            x_rotation: Angle { radians: 0.0 },
        })
        .collect();
    s.draw_arcs(&arcs)
}

#[inline]
pub(crate) fn fill_path<S: Surface + ?Sized>(s: &mut S, path: PathSlice<'_>) -> crate::Result {
    let points: Vec<Point<f32>> = path_utils::path_to_points(path.iter()).collect();
    s.fill_points(&points)
}

#[inline]
pub(crate) fn fill_path_buffer<S: Surface + ?Sized>(
    s: &mut S,
    buffer: PathBufferSlice<'_>,
) -> crate::Result {
    buffer.iter().try_for_each(|path| s.fill_path(path))
}

#[inline]
pub(crate) fn fill_rectangle<S: Surface + ?Sized>(
    s: &mut S,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> crate::Result {
    let mut builder = Builder::with_capacity(4, 4);
    build_rectangle(&mut builder, x, y, width, height);
    let path = builder.build();

    s.fill_path_owned(path)
}

#[inline]
pub(crate) fn fill_rectangles<S: Surface + ?Sized>(
    s: &mut S,
    rects: &[Rect<f32>],
) -> crate::Result {
    let mut buffer = PathBuffer::with_capacity(rects.len() * 4, rects.len() * 4, rects.len());
    rects.iter().copied().for_each(
        |Rect {
             origin: Point { x, y, .. },
             size: Size { width, height, .. },
         }| {
            let mut builder = buffer.builder();
            build_rectangle(&mut builder, x, y, width, height);
            builder.build();
        },
    );

    s.fill_path_buffer_owned(buffer)
}

#[inline]
pub(crate) fn fill_arc<S: Surface + ?Sized>(
    s: &mut S,
    xcenter: f32,
    ycenter: f32,
    xradius: f32,
    yradius: f32,
    start_angle: Angle<f32>,
    sweep_angle: Angle<f32>,
) -> crate::Result {
    let approx_perimeter_times_3 = (((xcenter + ycenter) * std::f32::consts::PI) as usize) << 2;

    let mut builder = Builder::with_capacity(approx_perimeter_times_3, approx_perimeter_times_3);
    build_arc(
        &mut builder,
        xcenter,
        ycenter,
        xradius,
        yradius,
        start_angle,
        sweep_angle,
    );
    let path = builder.build();

    s.fill_path_owned(path)
}

#[inline]
pub(crate) fn fill_arcs<S: Surface + ?Sized>(s: &mut S, arcs: &[Arc<f32>]) -> crate::Result {
    let mut buffer = PathBuffer::with_capacity(arcs.len(), arcs.len(), arcs.len());
    arcs.iter().copied().for_each(
        |Arc {
             center:
                 Point {
                     x: xcenter,
                     y: ycenter,
                     ..
                 },
             radii:
                 Vector {
                     x: xradius,
                     y: yradius,
                     ..
                 },
             start_angle,
             sweep_angle,
             ..
         }| {
            let mut builder = buffer.builder();
            build_arc(
                &mut builder,
                xcenter,
                ycenter,
                xradius,
                yradius,
                start_angle,
                sweep_angle,
            );
            builder.build();
        },
    );

    s.fill_path_buffer_owned(buffer)
}

#[inline]
pub(crate) fn fill_ellipses<S: Surface + ?Sized>(s: &mut S, ellipses: &[Ellipse]) -> crate::Result {
    let arcs: Vec<Arc<f32>> = ellipses
        .iter()
        .copied()
        .map(|Ellipse { center, radii }| Arc {
            center,
            radii,
            start_angle: Angle { radians: 0.0 },
            sweep_angle: Angle {
                radians: std::f32::consts::PI * 2.0,
            },
            x_rotation: Angle { radians: 0.0 },
        })
        .collect();
    s.fill_arcs(&arcs)
}

#[inline]
fn build_rectangle<B: PathBuilder>(builder: &mut B, x: f32, y: f32, width: f32, height: f32) {
    builder.begin(Point::new(x, y));
    builder.line_to(Point::new(x + width, y));
    builder.line_to(Point::new(x + width, y + height));
    builder.line_to(Point::new(x, y + height));
    builder.close();
}

#[inline]
fn build_arc<B: PathBuilder>(
    builder: &mut B,
    xcenter: f32,
    ycenter: f32,
    xradius: f32,
    yradius: f32,
    start_angle: Angle<f32>,
    sweep_angle: Angle<f32>,
) {
    let full_circle = std::f32::consts::PI * 2.0;
    let mut points = Arc {
        center: Point::new(xcenter, ycenter),
        radii: Vector::new(xradius, yradius),
        start_angle,
        sweep_angle,
        x_rotation: Angle { radians: 0.0 },
    }
    .flattened(0.334);
    if let Some(begin) = points.next() {
        builder.begin(begin);
        points.for_each(|pt| {
            builder.line_to(pt);
        });

        let divided = sweep_angle.radians / full_circle;

        // is this close to a multiple of 2*pi?
        builder.end(approx::abs_diff_eq!(divided.floor(), divided));
    }
}

#[inline]
fn draw_path_events<S: Surface + ?Sized, I: IntoIterator<Item = PathEvent>>(
    s: &mut S,
    iter: I,
) -> crate::Result {
    let lines: Vec<LineSegment<f32>> = path_utils::path_to_lines(iter).collect();

    s.draw_lines(&lines)
}

#[inline]
fn bezier_events(bezier: CubicBezierSegment<f32>) -> [PathEvent; 3] {
    let CubicBezierSegment {
        from,
        ctrl1,
        ctrl2,
        to,
    } = bezier;

    [
        PathEvent::Begin { at: from },
        PathEvent::Cubic {
            from,
            ctrl1,
            ctrl2,
            to,
        },
        PathEvent::End {
            last: to,
            first: from,
            close: false,
        },
    ]
}
