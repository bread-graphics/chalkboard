// MIT/Apache2 License

use ordered_float::NotNan;
use std::ops;

/// An angle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Angle {
    radians: NotNan<f32>,
}

impl Angle {
    pub const ZERO: Angle = unsafe { Angle::from_radians_unchecked(0.0) };
    pub const QUARTER_CIRCLE: Angle = unsafe { Angle::from_radians_unchecked(1.5707963267948966) };
    pub const HALF_CIRCLE: Angle = unsafe { Angle::from_radians_unchecked(std::f32::consts::PI) };
    pub const THREE_QUARTERS_CIRCLE: Angle =
        unsafe { Angle::from_radians_unchecked(4.71238898038469) };
    pub const FULL_CIRCLE: Angle = unsafe { Angle::from_radians_unchecked(6.283185307179586) };

    /// Create an angle based on the number of radians in the angle.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if the radians passed in is equal to NaN.
    #[inline]
    pub const unsafe fn from_radians_unchecked(radians: f32) -> Self {
        Self {
            radians: unsafe { NotNan::unchecked_new(radians) },
        }
    }

    /// Create an angle based on the number of radians in the angle. This function returns `None` if the radians given
    /// is NaN.
    #[inline]
    pub const fn from_radians(radians: f32) -> Option<Self> {
        // easy, const way to figure out if we are NaN: NaN is not equal to itself
        if radians != radians {
            None
        } else {
            Some(unsafe { Self::from_radians_unchecked(radians) })
        }
    }

    /// Create an angle based on the number of degrees in the angle.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if the radians passed in times pi divided by 180 is equal to NaN.
    #[inline]
    pub unsafe fn from_degrees_unchecked(degrees: f32) -> Self {
        unsafe { Self::from_radians_unchecked(degrees * std::f32::consts::PI * (1.0 / 180.0)) }
    }

    /// Create an angle based on the number of degrees in the angle. This function returns `None` if the degrees given
    /// times pi divided by 180 is NaN.
    #[inline]
    pub fn from_degrees(degrees: f32) -> Option<Self> {
        Self::from_radians(degrees * std::f32::consts::PI * (1.0 / 180.0))
    }

    /// Get the number of radians in this angle.
    #[inline]
    pub fn radians(self) -> f32 {
        self.radians.into_inner()
    }

    /// Get the number of degrees in this angle.
    #[inline]
    pub fn degrees(self) -> f32 {
        self.radians() * std::f32::consts::FRAC_1_PI * 180.0
    }

    /// Is this approximately equal to another angle?
    #[cfg(feature = "approx")]
    pub(crate) fn approx_eq(self, other: Self) -> bool {
        approx::abs_diff_eq!(self.radians(), other.radians())
    }
}

impl ops::Add for Angle {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            radians: self.radians + other.radians,
        }
    }
}
