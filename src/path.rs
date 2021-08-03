// MIT/Apache2 License

use lyon_geom::{point, Arc, CubicBezierSegment, LineSegment};
use lyon_path::{iterator::PathIterator, Path, PathEvent};
use std::array::IntoIter as ArrayIter;

/// Simple combinator to turn a path into a lines.
#[inline]
pub(crate) fn path_to_lines(
    i: impl IntoIterator<Item = PathEvent>,
) -> impl Iterator<Item = LineSegment<f32>> {
    i.into_iter().flattened().filter_map(|pe| match pe {
        PathEvent::Begin { .. } => None,
        PathEvent::Line { from, to } => Some(LineSegment { from, to }),
        PathEvent::End { last, first, close } => {
            if close {
                Some(LineSegment {
                    from: last,
                    to: first,
                })
            } else {
                None
            }
        }
        _ => unreachable!(),
    })
}

#[inline]
pub(crate) fn path_to_points(
    i: impl IntoIterator<Item = PathEvent>,
) -> impl Iterator<Item = Point<f32>> {
    i.into_iter().flattened().map(|pe| match pe {
        PathEvent::Begin { at } => at,
        PathEvent::Line { to, .. } => to,
        PathEvent::End { last, .. } => last,
        _ => unreachable!(),
    })
}

#[inline]
pub(crate) fn path_from_curve(curve: CubicBezierSegment<f32>) -> Path {
    let mut builder = Path::builder();
    builder.begin(curve.from);
    builder.cubic_bezier_to(curve.ctrl1, curve.ctrl2, curve.end);
    builder.end(false);
    builder.build()
}

#[inline]
pub(crate) fn path_from_arc(arc: Arc<f32>) -> Option<Path> {
    let mut builder = Path::builder();
    let mut iter = arc.flattened(1.0);
    builder.begin(iter.next()?);

    let mut builder = arc.flattened(1.0).fold(builder, |mut builder, point| {
        builder.line_to(point);
        builder
    });

    builder.end(false);
    Some(builder.build())
}

#[inline]
pub(crate) fn path_from_arc_closed(arc: Arc<f32>) -> Option<Path> {}
