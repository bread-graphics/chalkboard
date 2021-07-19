// MIT/Apache2 License

#![allow(clippy::excessive_precision, clippy::unreadable_literal)]

use core::ops;
use ordered_float::NotNan;

/// An angle, or a measure of the space between two intersecting lines.
///
/// Angles start from the bottom of the first quadrant and move counter-clockwise, just like angles in calculus
/// do.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Angle {
    radians: NotNan<f32>,
}

impl Angle {
    pub const ZERO: Angle = unsafe { Angle::from_radians_unchecked(0.0) };
    pub const QUARTER_CIRCLE: Angle =
        unsafe { Angle::from_radians_unchecked(core::f32::consts::FRAC_PI_2) };
    pub const HALF_CIRCLE: Angle = unsafe { Angle::from_radians_unchecked(core::f32::consts::PI) };
    pub const THREE_QUARTERS_CIRCLE: Angle =
        unsafe { Angle::from_radians_unchecked(4.71238898038469) };
    pub const FULL_CIRCLE: Angle = unsafe { Angle::from_radians_unchecked(6.283185307179586) };

    /// Create an angle based on the number of radians in the angle.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if the radians passed in is equal to NaN.
    #[allow(unused_unsafe)]
    #[must_use]
    #[inline]
    pub const unsafe fn from_radians_unchecked(radians: f32) -> Angle {
        Angle {
            radians: unsafe { NotNan::new_unchecked(radians) },
        }
    }

    /// Create an angle based on the number of radians in the angle. This function returns `None` if the radians
    /// given is NaN.
    #[must_use]
    #[inline]
    pub fn from_radians(radians: f32) -> Option<Angle> {
        if radians.is_nan() {
            None
        } else {
            Some(unsafe { Angle::from_radians_unchecked(radians) })
        }
    }

    /// Create an angle based on the number of degrees in the angle.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if the radians passed in times pi divided by 180 is equal to NaN.
    #[allow(unused_unsafe)]
    #[must_use]
    #[inline]
    pub unsafe fn from_degrees_unchecked(degrees: f32) -> Angle {
        unsafe { Angle::from_radians_unchecked(degrees * core::f32::consts::PI * (1.0 / 180.0)) }
    }

    /// Create an angle based on the number of degrees in the angle. This function returns `None` if the degrees
    /// given times pi divided by 180 is NaN.
    #[must_use]
    #[inline]
    pub fn from_degrees(degrees: f32) -> Option<Angle> {
        Angle::from_radians(degrees * core::f32::consts::PI * (1.0 / 180.0))
    }

    /// Get the number of radians in this angle.
    #[must_use]
    #[inline]
    pub fn radians(self) -> f32 {
        self.radians.into_inner()
    }

    /// Get the number of degrees in this angle.
    #[must_use]
    #[inline]
    pub fn degrees(self) -> f32 {
        self.radians() * core::f32::consts::FRAC_1_PI * 180.0
    }
}

impl ops::Add for Angle {
    type Output = Angle;

    #[inline]
    fn add(self, other: Angle) -> Angle {
        Angle {
            radians: self.radians + other.radians,
        }
    }
}
