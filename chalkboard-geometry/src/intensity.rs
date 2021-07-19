// MIT/Apache2 License

use core::ops;
use num_traits::{AsPrimitive, Bounded};
use ordered_float::NotNan;

/// A popular concept in is a range that goes from zero to one, defining intensity of a color or the
/// stop of a color in a `gradient. This type is essentially a wrapper around an `f32`, but with two invariants:
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
    #[allow(unused_unsafe)]
    #[must_use]
    #[inline]
    pub const unsafe fn new_unchecked(inner: f32) -> Intensity {
        Intensity {
            inner: unsafe { NotNan::new_unchecked(inner) },
        }
    }

    /// Create a new `Intensity`. If the inner value does not meet the invariants mentioned above, this function
    /// returns `None`.
    #[must_use]
    #[inline]
    pub fn new(inner: f32) -> Option<Intensity> {
        if inner.is_nan() || inner < 0.0 || inner > 1.0 {
            None
        } else {
            Some(Intensity {
                inner: unsafe { NotNan::new_unchecked(inner) },
            })
        }
    }

    /// Get the inner value of the `Intensity`.
    #[must_use]
    #[inline]
    pub fn into_inner(self) -> f32 {
        self.inner.into_inner()
    }

    /// Clamp this value to a compatible integer value.
    #[must_use]
    #[inline]
    pub fn clamp<N: Bounded + Copy + ops::Sub + 'static>(self) -> N
    where
        f32: AsPrimitive<N> + From<N::Output>,
    {
        let bounds: f32 = (N::max_value() - N::min_value()).into();
        (bounds * self.into_inner()).as_()
    }

    /// Clamp this value to a `u8`.
    #[must_use]
    #[inline]
    pub fn clamp_u8(self) -> u8 {
        self.clamp()
    }

    /// Clamp this value to a `u16`.
    #[must_use]
    #[inline]
    pub fn clamp_u16(self) -> u16 {
        self.clamp()
    }
}

impl From<Intensity> for f32 {
    #[inline]
    fn from(i: Intensity) -> f32 {
        i.into_inner()
    }
}
