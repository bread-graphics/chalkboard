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

use crate::{Line, Point2D as Point};
use core::{cmp, ops};
use num_traits::Zero;

/// The slope of a line.
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct Slope<Num> {
    /// The change in X.
    pub dx: Num,
    /// The change in Y.
    pub dy: Num,
}

impl<Num> Slope<Num> {
    /// Create a slope from two points.
    pub fn from_points<In: ops::Sub<Output = Num>>(pt1: Point<In>, pt2: Point<In>) -> Self {
        Slope {
            dx: pt1.x - pt2.x,
            dy: pt1.y - pt2.y,
        }
    }

    /// Create a slope from a `Line`.
    pub fn from_line<In: Copy + ops::Sub<Output = Num>>(line: Line<In>) -> Self {
        Slope::from_points(line.point, line.vector.to_point())
    }
}

impl<Num2, Num1: PartialEq<Num2>> PartialEq<Slope<Num2>> for Slope<Num1> {
    fn eq(&self, other: &Slope<Num2>) -> bool {
        self.dx == other.dx && self.dy == other.dy
    }
}

impl<Num: Eq> Eq for Slope<Num> {}

impl<DivResult, MulResult, Num1, Num2> PartialOrd<Slope<Num2>> for Slope<Num1>
where
    DivResult: PartialOrd + Zero,
    MulResult: PartialOrd,
    Num2: Clone + PartialOrd + Zero,
    Num1: Clone
        + PartialEq<Num2>
        + PartialOrd
        + ops::Mul<Num2, Output = MulResult>
        + ops::Div<Num2, Output = DivResult>
        + Zero,
{
    fn partial_cmp(&self, other: &Slope<Num2>) -> Option<cmp::Ordering> {
        use cmp::Ordering::*;

        // first, compare the cross-product of the slopes.
        let cross_product_1 = self.dy.clone() * other.dx.clone();
        let cross_product_2 = self.dx.clone() * other.dy.clone();
        if let x @ Less | x @ Greater = cross_product_2.partial_cmp(&cross_product_1)? {
            return Some(x);
        }

        // then, test for zero vectors
        let self_is_zero = self.dx == Num1::zero() && self.dy == Num1::zero();
        let other_is_zero = other.dx == Num2::zero() && other.dy == Num2::zero();

        match (self_is_zero, other_is_zero) {
            (true, true) => return Some(Equal),
            (true, false) => return Some(Greater),
            (false, true) => return Some(Less),
            (false, false) => {}
        }

        // these two slopes are either exactly the same of differ by
        // exactly pi
        //
        // to differentiate, look for a change in signage in either dx
        // or dy
        let sign_change_x = self.dx.clone() / other.dx.clone();
        let sign_change_y = self.dy.clone() / other.dy.clone();

        if sign_change_x < DivResult::zero() || sign_change_y < DivResult::zero() {
            // determine which one is the lesser
            if self.dx < Num1::zero() || (Num1::is_zero(&self.dx) && self.dy < Num1::zero()) {
                return Some(Less);
            } else {
                return Some(Greater);
            }
        }

        // all tests have failed, so the slopes have to be equal
        Some(Equal)
    }
}

impl<DivResult, MulResult, Num> Ord for Slope<Num>
where
    DivResult: Ord + Zero,
    MulResult: Ord,
    Num: Clone + Ord + ops::Mul<Output = MulResult> + ops::Div<Output = DivResult> + Zero,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
