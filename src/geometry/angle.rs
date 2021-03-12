// MIT/Apache2 License

use ordered_float::NotNan;

/// An angle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Angle {
    radians: NotNan<f32>,
}

impl Angle {
    pub const ZERO: Angle = unsafe { Angle::from_radians_unchecked(0.0) };
    pub const QUARTER_CIRCLE: Angle = unsafe { Angle::from_radians_unchecked(1.5707963267948966) };
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
        // pi/180 = 0.017453292519943295
        unsafe { Self::from_radians_unchecked(degrees * 0.017453292519943295) }
    }

    /// Create an angle based on the number of degrees in the angle. This function returns `None` if the degrees given
    /// times pi divided by 180 is NaN.
    #[inline]
    pub fn from_degrees(degrees: f32) -> Option<Self> {
        Self::from_radians(degrees * 0.017453292519943295)
    }

    /// Get the number of radians in this angle.
    #[inline]
    pub fn radians(self) -> f32 {
        self.radians.into_inner()
    }
}
