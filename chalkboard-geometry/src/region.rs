// BSL 1.0 License

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
        for box_ in iter {
            self.add(box_);
        }
    }
}

impl<T: Copy + Ord> Region<T> {
    /// Add new bounds to accomodate a box.
    fn accomodate(&mut self, box_: Box2D<T>) {
        if box_.min.x < self.bounds.min.x {
            self.bounds.min.x = box_.min.x;
        }

        if box_.min.y < self.bounds.min.y {
            self.bounds.min.y = box_.min.y;
        }

        if box_.max.x > self.bounds.max.x {
            self.bounds.max.x = box_.max.x;
        }

        if box_.max.y > self.bounds.max.y {
            self.bounds.max.y = box_.max.y;
        }
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
