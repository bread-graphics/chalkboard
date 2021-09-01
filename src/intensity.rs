// MIT/Apache2 License

use num_traits::{AsPrimitive, Bounded};
use ordered_float::NotNan;
use std::ops::Sub;

/// The intensity of a given phenomenon, given as a certain percentage.
///
/// It is useful to have a metric of saying "on a scale from `x` to `y`, how much of `z` is something?` For
/// instance, "how red is this color?" In this case, `Intensity` struct represents a floating point value
/// between `0.0` and `1.0`, where `0.0` is no red and `1.0` is as red as it can get. This is useful in other
/// cases as well; for instance, where the stops in a color gradient are.
///
/// The contained `f32` has two invariants:
///
/// * It cannot be `NaN`.
/// * It must be in the range `0.0..=1.0`.
///
/// Since constructing an `Intensity` is `unsafe` unless these two invariants are known to be settled, these
/// invariants can be relied on in unsafe code.
///
/// # Clamping
///
/// Let's take the colors again. It is common for colors to be representing using a byte per channel, where
/// `0` is the least red something can get and `255` is the most red. Note that these would be equal to `0.0`
/// and `1.0` in terms of the `Intensity` struct. Thus, it is often useful to convert this scale of `Intensity`
/// to the scale of actual numbers.
///
/// Using the `clamp` method, this dream can become reality. `clamp` takes a generic parameter: the type whose
/// limits you want the `Intensity` to clamp to. This type must implement the following traits:
///
/// * [`Bounded`]
/// * [`Into<f32>`]
/// * [`Sub`]
///
/// In addition, `f32` must also implement [`AsPrimitive<T>`].
///
/// [`AsPrimitive<T>`]: https://docs.rs/num-traits/*/num_traits/cast/trait.AsPrimitive.html
/// [`Bounded`]: https://docs.rs/num-traits/*/num_traits/bounds/trait.Bounded.html
/// [`Into<f32>`]: https://doc.rust-lang.org/1.54.0/std/convert/trait.Into.html
/// [`Sub`]: https://doc.rust-lang.org/1.54.0/std/ops/trait.Sub.html
///
/// ## Example
///
/// ```rust
/// use chalkboard::Intensity;
///
/// let lo = Intensity::new(0.0).unwrap();
/// let hi = Intensity::new(1.0).unwrap();
/// let middle = Intensity::new(0.5).unwrap();
/// assert_eq!(lo.clamp::<u8>(), 0);
/// assert_eq!(hi.clamp::<u8>(), 255);
///
/// let mc = middle.clamp::<u8>();
/// assert!(mc == 127 || mc = 128);
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Intensity {
    value: NotNan<f32>,
}

impl Intensity {
    /// Creates a new `Intensity` from an `f32` without checking the value.
    ///
    /// # Safety
    ///
    /// If `value` is not between `0.0` and `1.0` inclusive, or if `value` is `NaN`, the results of this
    /// function are undefined.
    #[inline]
    pub unsafe fn new_unchecked(value: f32) -> Intensity {
        Intensity {
            value: unsafe { NotNan::new_unchecked(value) },
        }
    }

    /// Creates a new `Intensity` from an `f32`.
    ///
    /// If `value` is not between `0.0` and `1.0` inclusive, or if `value` is `NaN`, this function will
    /// return `None`.
    #[inline]
    pub fn new(value: f32) -> Option<Intensity> {
        if ((0.0..=1.0).contains(&value)
            || approx::abs_diff_eq!(value, 1.0)
            || approx::abs_diff_eq!(value, 0.0))
            && !value.is_nan()
        {
            // SAFETY: invariants have been satisfied
            Some(unsafe { Intensity::new_unchecked(value) })
        } else {
            None
        }
    }

    /// Gets the inner value.
    #[inline]
    pub fn into_inner(self) -> f32 {
        self.value.into_inner()
    }

    /// Clamp this intensity to a value.
    ///
    /// This represents the `Intensity` as a value between `T::MIN` and `T::MAX`. See the struct-level
    /// documentation for more information.
    #[inline]
    pub fn clamp<T: 'static + Bounded + Copy + Sub>(self) -> T
    where
        f32: AsPrimitive<T> + From<T::Output>,
    {
        let scale = T::max_value() - T::min_value();
        let scale: f32 = scale.into();
        let value = self.into_inner() * scale;
        value.as_()
    }
}
