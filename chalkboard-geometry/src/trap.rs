// This file is part of chalkboard.
//
// chalkboard is free software: you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation,
// either version 3 of the License, or (at your option)
// any later version.
//
// chalkboard is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty
// of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General
// Public License along with chalkboard. If not, see
// <https://www.gnu.org/licenses/>.

use core::{iter::FusedIterator, ops::Sub, ptr::eq};
use lyon_geom::Scalar;
use num_traits::{Float, Zero};

use crate::{thrice::Thrice, util::approx_eq};

use super::{Box2D, Line, Point2D, Rect, Triangle, Vector2D};

/// A two-dimensional trapezoid.
///
/// This trapezoid is defined by two edges, one on each side of the
/// trapezoid. The top and bottom edges of the trapezoid are horizontal,
/// and are defined by the `top` and `bottom` fields.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Trapezoid<T> {
    /// The upper limit of the trapezoid.
    pub top: T,
    /// The lower limit of the trapezoid.
    pub bottom: T,
    /// The left side of the trapezoid.
    pub left: Line<T>,
    /// The right side of the trapezoid.
    pub right: Line<T>,
}

impl<T> Trapezoid<T> {
    /// Create a new trapezoid from its core components.
    pub const fn new(top: T, bottom: T, left: Line<T>, right: Line<T>) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    /// Create a new trapezoid from a `Box2D`.
    pub fn from_box(value: Box2D<T>) -> Self
    where
        T: Clone + Sub<Output = T> + Zero,
    {
        let Point2D { y: min_y, .. } = value.min.clone();

        let tl = value.min;
        let tr = Point2D::new(value.max.x, min_y.clone());
        let vector = Vector2D::new(T::zero(), value.max.y.clone() - min_y.clone());

        Self::new(
            min_y,
            value.max.y,
            Line {
                point: tl,
                vector: vector.clone(),
            },
            Line { point: tr, vector },
        )
    }

    /// Create a new trapezoid from a `Rect`.
    pub fn from_rect(value: Rect<T>) -> Self
    where
        T: Copy + Sub<Output = T> + Zero,
    {
        let b = Box2D::from_origin_and_size(value.origin, value.size);

        Self::from_box(b)
    }

    /// Create a set of `Trapezoid`s from a triangle.
    ///
    /// This will produce, at most, two trapezoids.
    pub fn from_triangle(
        triangle: Triangle<T>,
    ) -> impl FusedIterator<Item = Self> + ExactSizeIterator + DoubleEndedIterator
    where
        T: Scalar,
    {
        // if two of three triangle's points have an equal Y coordinate,
        // then the triangle can be represented as a single trapezoid
        match (
            approx_eq(triangle.a.y, triangle.b.y),
            approx_eq(triangle.b.y, triangle.c.y),
            approx_eq(triangle.c.y, triangle.a.y),
        ) {
            (true, true, true) => return Thrice::empty(),
            (true, false, false) => {
                return Thrice::one(Self::tri(triangle.c, triangle.a, triangle.b))
            }
            (false, true, false) => {
                return Thrice::one(Self::tri(triangle.a, triangle.b, triangle.c))
            }
            (false, false, true) => {
                return Thrice::one(Self::tri(triangle.b, triangle.c, triangle.a))
            }
            _ => {}
        }

        // order the triangle points by Y into top, middle, and bottom
        let mut pts = [triangle.a, triangle.b, triangle.c];
        pts.sort_unstable_by(|pt1, pt2| pt1.y.partial_cmp(&pt2.y).unwrap());

        // there will be a fourth point, at Y = middle.y, which is used
        // to split the triangle into 2 trapezoids
        let [top, middle, bottom] = pts;
        let tb_line = Line {
            point: top,
            vector: bottom - top,
        };
        let divider_x = tb_line
            .equation()
            .solve_x_for_y(middle.y)
            .expect("the line should never be horizontal");
        let divider = Point2D::new(divider_x, middle.y);

        Thrice::two(
            Self::tri(top, middle, divider),
            Self::tri(bottom, middle, divider),
        )
    }

    /// Create a trapezoid from three points, where two points have an
    /// equal Y coordinate.
    fn tri(off_pt: Point2D<T>, eq_pt1: Point2D<T>, eq_pt2: Point2D<T>) -> Self
    where
        T: Scalar,
    {
        let (top, bottom) = if off_pt.y < eq_pt1.y {
            (off_pt, eq_pt1)
        } else {
            (eq_pt1, off_pt)
        };

        let (left_pt, right_pt) = if eq_pt1.x < eq_pt2.x {
            (eq_pt1, eq_pt2)
        } else {
            (eq_pt2, eq_pt1)
        };

        let left_line = Line {
            point: off_pt,
            vector: left_pt - off_pt,
        };
        let right_line = Line {
            point: off_pt,
            vector: right_pt - off_pt,
        };

        Self::new(top.y, bottom.y, left_line, right_line)
    }
}
