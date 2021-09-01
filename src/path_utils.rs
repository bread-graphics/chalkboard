// MIT/Apache2 License

use lyon_geom::{LineSegment, Point};
use lyon_path::{iterator::PathIterator, PathEvent};

#[inline]
pub(crate) fn path_to_lines<I: IntoIterator<Item = PathEvent>>(
    iter: I,
) -> impl Iterator<Item = LineSegment<f32>> {
    iter.into_iter()
        .flattened(0.5)
        .filter_map(|event| match event {
            PathEvent::Line { from, to } => Some(LineSegment { from, to }),
            PathEvent::End {
                last,
                first,
                close: true,
            } => Some(LineSegment {
                from: last,
                to: first,
            }),
            _ => None,
        })
}

#[inline]
pub(crate) fn path_to_points<I: IntoIterator<Item = PathEvent>>(
    iter: I,
) -> impl Iterator<Item = Point<f32>> {
    iter.into_iter()
        .flattened(0.5)
        .filter_map(|event| match event {
            PathEvent::Begin { at } => Some(at),
            PathEvent::Line { to, .. } => Some(to),
            _ => None,
        })
}
