// MIT/Apache2 License

use crate::util::clamp;
use ordered_float::NotNan;

/// A popular concept in chalkboard is a range that goes from zero to one, defining intensity of a color or the
/// stop of a color in a gradient. This type is essentially a wrapper around an `f32`, but with two invariants:
///
/// * The inner value will always be between `0.0` and `1.0`.
/// * The inner value will never be `NaN`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Intensity {
    inner: NotNan<f32>,
}

impl Intensity {
    /// Create a new `Intensity`, without checking the inner value.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if `inner` is not a number, or outside of the range [0, 1].
    #[inline]
    pub const unsafe fn new_unchecked(inner: f32) -> Self {
        Self {
            inner: unsafe { NotNan::unchecked_new(inner) },
        }
    }

    /// Create a new `Intensity`. If the inner value does not meet the invariants mentioned above, this function
    /// returns `None`.
    #[inline]
    pub fn new(inner: f32) -> Option<Self> {
        if inner.is_nan() || inner < 0.0 || inner > 1.0 {
            None
        } else {
            Some(Self {
                inner: unsafe { NotNan::unchecked_new(inner) },
            })
        }
    }

    /// Get the inner value of the `Intensity`.
    #[inline]
    pub fn into_inner(self) -> f32 {
        self.inner.into_inner()
    }

    /// Clamp this value to a `u8`.
    #[inline]
    pub fn clamp_to_u8(self) -> u8 {
        clamp(self.into_inner())
    }

    /// Clamp this value to a `u16`.
    #[inline]
    pub fn clamp_to_u16(self) -> u16 {
        clamp(self.into_inner())
    }
}

impl From<Intensity> for f32 {
    #[inline]
    fn from(i: Intensity) -> f32 {
        i.into_inner()
    }
}
