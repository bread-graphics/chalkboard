//               Copyright John Nunley, 2022.
// Distributed under the Boost Software License, Version 1.0.
//       (See accompanying file LICENSE or copy at
//         https://www.boost.org/LICENSE_1_0.txt)

use core::iter::FromIterator;

use super::{Box2D, Point2D};
use alloc::vec::Vec;
use num_traits::Bounded;

/// A region covering a certain area.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Region<T> {
    /// The bounding box of the region.
    bounds: Box2D<T>,
    /// The list of boxes that make up the region.
    boxes: Vec<Box2D<T>>,
}

impl<T: Bounded> Default for Region<T> {
    fn default() -> Self {
        Self {
            bounds: default_bounds(),
            boxes: alloc::vec![],
        }
    }
}

impl<T: Bounded + Copy + Ord> FromIterator<Box2D<T>> for Region<T> {
    fn from_iter<I: IntoIterator<Item = Box2D<T>>>(iter: I) -> Self {
        let mut region = Self::default();
        region.extend(iter);
        region
    }
}

impl<T: Copy + Ord> Extend<Box2D<T>> for Region<T> {
    fn extend<I: IntoIterator<Item = Box2D<T>>>(&mut self, iter: I) {
        let bounds = &mut self.bounds;
        let boxes = &mut self.boxes;

        boxes.extend(iter.into_iter().inspect(|box_| add_to_bounds(bounds, box_)));
    }
}

impl<T: Copy + Ord> Region<T> {
    /// Add new bounds to accomodate a box.
    fn accomodate(&mut self, box_: Box2D<T>) {
        add_to_bounds(&mut self.bounds, &box_)
    }

    /// Add a new `Box2D` to the region.
    pub fn add(&mut self, box_: Box2D<T>) {
        self.accomodate(box_);
        self.boxes.push(box_);
    }
}

impl<T: Copy> Region<T> {
    pub fn bounds(&self) -> Box2D<T> {
        self.bounds
    }

    pub fn boxes(&self) -> &[Box2D<T>] {
        &self.boxes
    }
}

fn default_bounds<T: Bounded>() -> Box2D<T> {
    Box2D::new(
        Point2D::new(T::min_value(), T::min_value()),
        Point2D::new(T::max_value(), T::max_value()),
    )
}

fn add_to_bounds<T: Clone + PartialOrd>(bounds: &mut Box2D<T>, new: &Box2D<T>) {
    if new.min.x < bounds.min.x {
        bounds.min.x = new.min.x.clone();
    }

    if new.min.y < bounds.min.y {
        bounds.min.y = new.min.y.clone();
    }

    if new.max.x > bounds.max.x {
        bounds.max.x = new.max.x.clone();
    }

    if new.max.y > bounds.max.y {
        bounds.max.y = new.max.y.clone();
    }
}
