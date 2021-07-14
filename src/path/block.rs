// MIT/Apache2 License

use crate::Point;
use super::{Path, PathSegment, PathSegmentType};

impl Path {
    /// Given a line width, converts this one into a polygonal representation.
    #[inline]
    pub(crate) fn into_polygon(self, line_width: usize) -> impl Iterator<Item = Point> {

    }
}
