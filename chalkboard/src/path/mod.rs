// MIT/Apache2 License

use crate::geometry::{BezierCurve, GeometricArc, Line, Point};
use std::{array::IntoIter as ArrayIter, ops, vec::IntoIter as VecIter};
use tinyvec::TinyVec;

/// A segment of a path.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathSegment {
    pub x: i32,
    pub y: i32,
    pub ty: PathSegmentType,
}

/// How the path segments connects to the next one.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PathSegmentType {
    /// No connection. Ends the path.
    Close,
    /// Straight line.
    StraightLine,
    /// Bezier curve line, with two control points.
    BezierCurve {
        ctx1: i32,
        cty1: i32,
        ctx2: i32,
        cty2: i32,
    },
}

/// A path. This is the elementary unit of shapes in this framework.
/// 
/// A path consists of a series of path segments, ended with a "close" path segment. This represents a line that
/// is not completely straight, and may contain curves or vertices. In other contexts, it may represent a closed
/// two-dimensional shape.
/// 
/// Although a path that intersects itself is valid for stroke functions, it is not valid for fill functions.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Path {
    segments: Vec<PathSegment>,
}

impl IntoIterator for Path {
    type Item = PathSegment;
    type IntoIter = VecIter<PathSegment>;

    #[inline]
    fn into_iter(self) -> VecIter<PathSegment> {
        self.segments.into_iter()
    }
}

impl Path {
    /// Closes this path with an additional straight line.
    /// 
    /// # Panics
    /// 
    /// Panics if the path is empty.
    #[inline]
    pub fn close(&mut self) {
        let first = self.segments.first().expect("No segments").clone();
        let last = self.segments.last_mut().expect("No segments");
        if first.x != last.x || first.y != last.y {
            last.ty = PathSegmentType::StraightLine;
            let new_seg = PathSegment {
                x: first.x,
                y: first.y,
                ty: PathSegmentType::Close,
            };
            self.segments.push(new_seg);
        }
    }

    /// Create a new path from a collection of path segments.
    #[inline]
    pub fn new<I: IntoIterator<Item = PathSegment>>(i: I) -> Self {
        let segments: Vec<PathSegment> = i.into_iter().collect();
        Self { segments }
    }

    /// Create a new path consisting of only a line.
    #[inline]
    pub fn from_line(x1: i32, y1: i32, x2: i32, y2: i32) -> Self {
        Self { segments: vec![
            PathSegment {
                x: x1,
                y: y1,
                ty: PathSegmentType::StraightLine,
            },
            PathSegment {
                x: x2,
                y: y2,
                ty: PathSegmentType::Close,
            },
        ] }
    }

    /// Create a new path consisting of only a bezier curve.
    #[inline]
    pub fn from_bezier_curve(bc: BezierCurve) -> Self {
        Self { segments: vec![
            PathSegment {
                x: bc.start.x,
                y: bc.start.y,
                ty: PathSegmentType::BezierCurve {
                    ctx1: bc.control1.x,
                    ctx2: bc.control2.x,
                    cty1: bc.control1.y,
                    cty2: bc.control2.y,
                },
            },
            PathSegment {
                x: bc.end.x,
                y: bc.end.y,
                ty: PathSegmentType::Close,
            },
        ] }
    }

    /// Create a new path consisting of only an arc.
    #[inline]
    pub fn from_arc(arc: GeometricArc) -> Self {
        Self::from_bezier_curve(arc.into_curve())
    }

    /// Convert this path into a series of points.
    #[inline]
    pub fn into_points(self) -> impl Iterator<Item = Point> {
        self.segments.into_iter().scan
    }

    /// Convert this path to a series of lines.
    #[inline]
    pub fn into_lines(self) -> Vec<Line> {
        let seglen = self.segments.len();
        self.segments
            .into_iter()
            .fold(
                (
                    PathSegment {
                        x: 0,
                        y: 0,
                        ty: PathSegmentType::Close,
                    },
                    Vec::with_capacity(seglen),
                ),
                |(prevseg, mut curlines), seg| {
                    match prevseg.ty {
                        // if the previous one was a closing segment, just replace it
                        PathSegmentType::Close => (seg, curlines),
                        // if the previous one was a straight line segment, add a new line
                        PathSegmentType::StraightLine => {
                            curlines.push(Line {
                                x1: prevseg.x,
                                y1: prevseg.y,
                                x2: seg.x,
                                y2: seg.y,
                            });
                            (seg, curlines)
                        }
                        // if the previous one was a bezier curve, add all the lines involved in the bezier curve
                        PathSegmentType::BezierCurve {
                            ctx1,
                            cty1,
                            ctx2,
                            cty2,
                        } => {
                            let curve = BezierCurve {
                                start: Point {
                                    x: prevseg.x,
                                    y: prevseg.y,
                                },
                                end: Point { x: seg.x, y: seg.y },
                                control1: Point { x: ctx1, y: cty1 },
                                control2: Point { x: ctx2, y: cty2 },
                            };
                            curlines.extend(curve.into_lines());
                            (seg, curlines)
                        }
                    }
                },
            )
            .1
    }
}
