// BSL 1.0 License

use super::Line;

/// A two-dimensional trapezoid.
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
    pub fn new(
        top: T,
        bottom: T,
        left: Line<T>,
        right: Line<T>,
    ) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }
}