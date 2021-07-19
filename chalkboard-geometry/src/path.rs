// MIT/Apache2 License

#![allow(clippy::module_name_repetitions)]

use super::{outline, polyline, BezierCurve, GeometricArc, Line, Point};
use core::{
    cmp,
    iter::{Fuse, FusedIterator},
};

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec, vec::Vec};
#[cfg(feature = "alloc")]
use core::{iter::FromIterator, ops, ptr};

/// An implemenation of a path. This represents an arbitrary line between two points, involving straight lines
/// and bezier curves. Alternatively, this may represent a closed path that may be able to be filled.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Path {
    segments: Vec<PathSegment>,
}

/// A slice of a [`Path`]. This object is to `Path` as `str` is to `String`, or `[T]` is to `Vec<T>`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PathSlice {
    segments: [PathSegment],
}

/// A segment in a path. This represents one point in a path, as well as the way this vertex leads to the next
/// point.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathSegment {
    /// The X coordinate of the path segment.
    pub x: i32,
    /// The Y coordinate of the path segment.
    pub y: i32,
    /// The way this vertex links to the next vertex. This value is meaningless if this is the last value in the
    /// path.
    pub ty: PathSegmentType,
}

/// The way that this segment of the path links to the next segment.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PathSegmentType {
    /// The next segment is linked via a straight line.
    StraightLine,
    /// The next segment is linked via a bezier curve, with the following control points.
    BezierCurve {
        ctx1: i32,
        cty1: i32,
        ctx2: i32,
        cty2: i32,
    },
}

impl Default for PathSegmentType {
    #[inline]
    fn default() -> PathSegmentType {
        PathSegmentType::StraightLine
    }
}

impl PathSlice {
    /// Convert a slice of [`PathSegment`]s to a `PathSlice`.
    #[must_use]
    #[inline]
    pub fn from_segment_slice(segments: &[PathSegment]) -> &PathSlice {
        // SAFETY: PathSlice is repr(transparent), so it has the same layout as a PathSegment slice. This cast is
        //         valid.
        unsafe { &*(segments as *const [PathSegment] as *const PathSlice) }
    }

    /// Convert a mutable slice of [`PathSegments`]s to a mutable `PathSlice`.
    #[inline]
    pub fn from_segment_slice_mut(segments: &mut [PathSegment]) -> &mut PathSlice {
        // SAFETY: same as above
        unsafe { &mut *(segments as *mut [PathSegment] as *mut PathSlice) }
    }

    /// Convert a boxed `PathSlice` into a `Path`.
    #[cfg(feature = "alloc")]
    #[must_use]
    #[inline]
    pub fn into_path(self: Box<PathSlice>) -> Path {
        // SAFETY: same as above
        let segments: Box<[PathSegment]> =
            unsafe { Box::from_raw(Box::into_raw(self) as *mut [PathSegment]) };
        let segments = segments.into_vec();
        Path { segments }
    }

    /// Copy this `PathSlice` into a `Path`.
    #[cfg(feature = "alloc")]
    #[must_use]
    #[inline]
    pub fn to_path(&self) -> Path {
        let len = self.segments.len();
        let mut path = Path {
            segments: Vec::with_capacity(len),
        };

        unsafe {
            ptr::copy_nonoverlapping(self.segments.as_ptr(), path.segments.as_mut_ptr(), len);
            path.segments.set_len(len);
        }

        path
    }

    /// The path is meaningless if there are no segments or if there is only one segment. A meaningless path will
    /// draw nothing to the screen, and is a no-op for several other operations as well.
    #[inline]
    #[must_use]
    pub fn is_meaningless(&self) -> bool {
        self.segments.len() <= 1
    }

    /// Get an iterator over the points that this `PathSlice` encompasses.
    #[inline]
    pub fn iter_points(&self) -> impl Iterator<Item = Point> + '_ {
        path_segments_to_points(self.segments.iter().copied())
    }

    /// Get an iterator over the lines that this `PathSlice` encompasses.
    #[inline]
    pub fn iter_lines(&self) -> impl Iterator<Item = Line> + '_ {
        polyline(self.iter_points())
    }

    /// Get a closed path representing an outline of this path.
    #[cfg(feature = "alloc")]
    #[must_use]
    #[inline]
    pub fn outline(&self, line_width: u32) -> Path {
        let mut path = Path::polyline(outline(self.iter_points(), line_width));
        path.close();
        path
    }
}

#[cfg(feature = "alloc")]
impl Path {
    /// Create a new, empty path.
    #[must_use]
    #[inline]
    pub fn new() -> Path {
        Path { segments: vec![] }
    }

    /// Create a path based upon a series of points forming a polyline.
    #[inline]
    pub fn polyline<I: IntoIterator<Item = Point>>(points: I) -> Path {
        Path {
            segments: points
                .into_iter()
                .map(|Point { x, y }| PathSegment {
                    x,
                    y,
                    ty: PathSegmentType::StraightLine,
                })
                .collect(),
        }
    }

    /// Create a path based around a `GeometricArc`, but closed.
    #[inline]
    pub fn from_arc_closed(arc: GeometricArc) -> Path {
        let GeometricArc { x1, y1, x2, y2, .. } = arc;
        let mut path = Path::from(arc);
        let xr = (x2 - x1).abs();
        let yr = (y2 - y1).abs();
        let xm = cmp::min(x1, x2);
        let ym = cmp::min(y1, y2);

        // if the arc is not a full ellipse, complete it
        path.push(PathSegment {
            x: xr + xm,
            y: yr + ym,
            ty: PathSegmentType::StraightLine,
        });
        path.close();

        path
    }

    /// Convert this `Path` into a boxed `PathSlice`.
    #[must_use]
    #[inline]
    pub fn into_boxed_path_slice(self) -> Box<PathSlice> {
        let segments: Box<[PathSegment]> = self.segments.into_boxed_slice();
        // SAFETY: [PathSegment] and PathSlice share the same layout
        unsafe { Box::from_raw(Box::into_raw(segments) as *mut [PathSegment] as *mut PathSlice) }
    }

    /// Get an iterator over the points that this `Path` encompasses.
    #[inline]
    pub fn into_iter_points(self) -> impl Iterator<Item = Point> {
        path_segments_to_points(self.segments.into_iter())
    }

    /// Get an iterator over the lines that this `Path` encompasses.
    #[inline]
    pub fn into_iter_lines(self) -> impl Iterator<Item = Line> {
        polyline(self.into_iter_points())
    }

    /// Convert this path into an equivalent closed `Path` representing an outline of this path.
    #[inline]
    pub fn into_outline(self, line_width: u32) -> Path {
        let mut path = Path::polyline(outline(self.into_iter_points(), line_width));
        path.close();
        path
    }

    /// Closes the path with a straight line between the last path segment in the list and the first one...
    /// unless the path is already closed, then this does nothing.
    ///
    /// If the path is meaningless, this does nothing.
    #[allow(clippy::missing_panics_doc)]
    #[inline]
    pub fn close(&mut self) {
        if self.is_meaningless() {
            return;
        }

        let first = *self.segments.first().unwrap();
        let last = self.segments.last_mut().unwrap();

        if first.x != last.x || first.y != last.y {
            last.ty = PathSegmentType::StraightLine;
            let new_seg = PathSegment {
                x: first.x,
                y: first.y,
                ty: PathSegmentType::StraightLine,
            };
            self.segments.push(new_seg);
        }
    }

    /// Add a segment to this path.
    #[inline]
    pub fn push(&mut self, seg: PathSegment) {
        self.segments.push(seg);
    }
}

#[cfg(feature = "alloc")]
impl From<BezierCurve> for Path {
    #[inline]
    fn from(bc: BezierCurve) -> Path {
        Path {
            segments: vec![
                PathSegment {
                    x: bc.start.x,
                    y: bc.start.y,
                    ty: PathSegmentType::BezierCurve {
                        ctx1: bc.control1.x,
                        cty1: bc.control1.y,
                        ctx2: bc.control2.x,
                        cty2: bc.control2.y,
                    },
                },
                PathSegment {
                    x: bc.end.x,
                    y: bc.end.y,
                    ty: PathSegmentType::StraightLine,
                },
            ],
        }
    }
}

#[cfg(feature = "alloc")]
impl From<GeometricArc> for Path {
    #[inline]
    fn from(arc: GeometricArc) -> Path {
        Path::polyline(arc.into_points())
    }
}

#[cfg(feature = "alloc")]
impl FromIterator<PathSegment> for Path {
    #[inline]
    fn from_iter<T: IntoIterator<Item = PathSegment>>(iter: T) -> Path {
        Path {
            segments: iter.into_iter().collect(),
        }
    }
}

#[cfg(feature = "alloc")]
impl ops::Deref for Path {
    type Target = PathSlice;

    #[inline]
    fn deref(&self) -> &PathSlice {
        PathSlice::from_segment_slice(&self.segments)
    }
}

#[cfg(feature = "alloc")]
impl ops::DerefMut for Path {
    #[inline]
    fn deref_mut(&mut self) -> &mut PathSlice {
        PathSlice::from_segment_slice_mut(&mut self.segments)
    }
}

/// Turn an iterator over a series of `PathSegment`s into a series of points.
#[inline]
fn path_segments_to_points<I: IntoIterator<Item = PathSegment>>(
    segments: I,
) -> impl Iterator<Item = Point> {
    PathSegmentsToPoints {
        inner_pathseg_iter: segments.into_iter().fuse(),
        front_bezier_curve_iter: None,
        back_bezier_curve_iter: None,
        bezier_curve_iteration_function: BezierCurve::into_points,
        front_prev_segment: None,
        back_prev_segment: None,
    }
}

// If there is a way to do this without a custom iterator, please open a pull request.
// This is sort of akin to a scan() followed by a flat_map(), but it's not possible without allocating, at
// least, as far as I know.
// This is also double-ended so it works with outline(), which scan() does not.
#[derive(Clone)]
struct PathSegmentsToPoints<I, B> {
    inner_pathseg_iter: Fuse<I>,
    front_bezier_curve_iter: Option<B>,
    back_bezier_curve_iter: Option<B>,
    bezier_curve_iteration_function: fn(BezierCurve) -> B,
    front_prev_segment: Option<PathSegment>,
    back_prev_segment: Option<PathSegment>,
}

impl<I, B> PathSegmentsToPoints<I, B> {
    #[inline]
    fn front_bezier_curve_iter(&mut self, from_front: bool) -> &mut Option<B> {
        if from_front {
            &mut self.front_bezier_curve_iter
        } else {
            &mut self.back_bezier_curve_iter
        }
    }

    #[inline]
    fn back_bezier_curve_iter(&mut self, from_front: bool) -> &mut Option<B> {
        if from_front {
            &mut self.back_bezier_curve_iter
        } else {
            &mut self.front_bezier_curve_iter
        }
    }

    #[inline]
    fn front_prev_segment(&mut self, from_front: bool) -> &mut Option<PathSegment> {
        if from_front {
            &mut self.front_prev_segment
        } else {
            &mut self.back_prev_segment
        }
    }

    #[inline]
    fn back_prev_segment(&mut self, from_front: bool) -> &mut Option<PathSegment> {
        if from_front {
            &mut self.back_prev_segment
        } else {
            &mut self.front_prev_segment
        }
    }

    #[inline]
    fn next_impl<
        FI: FnMut(&mut Fuse<I>) -> Option<PathSegment>,
        FB: FnMut(&mut B) -> Option<Point>,
    >(
        &mut self,
        from_front: bool,
        mut iter_next: FI,
        mut bezier_next: FB,
    ) -> Option<Point> {
        loop {
            // if there are any elements left in the ongoing bezier curve iterator, take them
            if let Some(bezier_curve_iter) = self.front_bezier_curve_iter(from_front).as_mut() {
                match bezier_next(bezier_curve_iter) {
                    Some(pt) => return Some(pt),
                    None => {
                        *self.front_bezier_curve_iter(from_front) = None;
                    }
                }
            }

            // get a segment from the inner iterator
            let seg = match iter_next(&mut self.inner_pathseg_iter) {
                Some(seg) => seg,
                None => {
                    // we may not have yet taken the latest prev_segment and returned its inner point
                    // the type is irrelevant here
                    // this does not defeat meaningless paths
                    // if that's empty, try to take from the existing back
                    match (
                        self.front_prev_segment(from_front).take(),
                        self.back_bezier_curve_iter(from_front).as_mut(),
                    ) {
                        (Some(PathSegment { x, y, .. }), _) => return Some(Point { x, y }),
                        // try to iterate from the back if possible
                        (None, Some(bezier_curve_iter)) => match bezier_next(bezier_curve_iter) {
                            Some(pt) => return Some(pt),
                            None => {}
                        },
                        (None, None) => {}
                    }

                    return self
                        .back_prev_segment(from_front)
                        .take()
                        .map(|PathSegment { x, y, .. }| Point { x, y });
                }
            };

            // if prev_segment has yet to be set, get an element from the inner iterator and set it
            let prev_seg = match self.front_prev_segment(from_front).replace(seg) {
                Some(prev_seg) => prev_seg,
                None => continue,
            };

            match prev_seg {
                PathSegment {
                    x,
                    y,
                    ty: PathSegmentType::StraightLine,
                } => {
                    // straight line starting from (x, y), just return
                    return Some(Point { x, y });
                }
                PathSegment {
                    x,
                    y,
                    ty:
                        PathSegmentType::BezierCurve {
                            ctx1,
                            cty1,
                            ctx2,
                            cty2,
                        },
                } => {
                    // create a bezier curve representing the path
                    let curve = BezierCurve {
                        start: Point { x, y },
                        control1: Point { x: ctx1, y: cty1 },
                        control2: Point { x: ctx2, y: cty2 },
                        end: Point { x: seg.x, y: seg.y },
                    };

                    // transform that into an iterator
                    let i = (self.bezier_curve_iteration_function)(curve);

                    // begin iterating over that
                    *self.front_bezier_curve_iter(from_front) = Some(i);
                }
            }
        }
    }
}

impl<I: Iterator<Item = PathSegment>, B: Iterator<Item = Point>> Iterator
    for PathSegmentsToPoints<I, B>
{
    type Item = Point;

    #[inline]
    fn next(&mut self) -> Option<Point> {
        self.next_impl(true, Fuse::next, B::next)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // because of the possibility of embedded bezier curves we can't give an exact length, but we can
        // give an estimate
        // the lower bound is the lower bound of the inner pathseg iterator, but minus one if the prev seg
        // is not yet established
        // we can't give an upper bound, as the bezier curve iteration contains an unpredictable number of
        // points
        let (mut lower, _) = self.inner_pathseg_iter.size_hint();
        if self.front_prev_segment.is_none() {
            lower = lower.saturating_sub(1);
        }
        if let Some(ref bci) = self.front_bezier_curve_iter {
            let (bci_lower, _) = bci.size_hint();
            lower = lower.saturating_add(bci_lower);
        }

        (lower, None)
    }
}

// Fuses no matter what.
impl<I: Iterator<Item = PathSegment>, B: Iterator<Item = Point>> FusedIterator
    for PathSegmentsToPoints<I, B>
{
}

impl<
        I: Iterator<Item = PathSegment> + DoubleEndedIterator,
        B: Iterator<Item = Point> + DoubleEndedIterator,
    > DoubleEndedIterator for PathSegmentsToPoints<I, B>
{
    #[inline]
    fn next_back(&mut self) -> Option<Point> {
        self.next_impl(false, Fuse::next_back, B::next_back)
    }
}
