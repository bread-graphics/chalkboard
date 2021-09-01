// MIT/Apache2 License

use super::Intensity;
use num_traits::{AsPrimitive, Bounded};
use std::ops::Sub;

/// Represents a four-channel RGBA color.
///
/// Things tend to have colors. You know this. `Color` aims to be a standard way of representing these colors
/// throughout the ecosystem. Insert your own XKCD 927 reference here.
///
/// `Color` is represented internally as a collection of four [`Intensity`] objects. This allows it to be
/// clamped similarly to how `Intensity` is. `Intensity` is represented as a non-`NaN` float between `0.0`
/// and `1.0`. The massive range that floating point values have allows this struct to theoretically represent
/// a similarly massive array of colors.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Color {
    pub red: Intensity,
    pub green: Intensity,
    pub blue: Intensity,
    pub alpha: Intensity,
}

impl Color {
    /// Creates a new color from four floats representing channels, without checking for validity.
    ///
    /// # Safety
    ///
    /// If any of the components are `NaN` or not in the range `0.0..=1.0`, behavior is undefined.
    #[inline]
    pub unsafe fn new_unchecked(red: f32, green: f32, blue: f32, alpha: f32) -> Color {
        unsafe {
            Color {
                red: Intensity::new_unchecked(red),
                green: Intensity::new_unchecked(green),
                blue: Intensity::new_unchecked(blue),
                alpha: Intensity::new_unchecked(alpha),
            }
        }
    }

    /// Creates a new color from four floats representing channels.
    ///
    /// If any of the components are `NaN` or not in the range `0.0..=1.0`, this function returns `None`.
    #[inline]
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Option<Color> {
        Some(Color {
            red: Intensity::new(red)?,
            green: Intensity::new(green)?,
            blue: Intensity::new(blue)?,
            alpha: Intensity::new(alpha)?,
        })
    }

    /// Clamps the components of this color into an array of four channels.
    ///
    /// See the `clamp` method on [`Intensity`] for more information on how clamping works.
    #[inline]
    pub fn clamp<T: 'static + Bounded + Copy + Sub>(self) -> [T; 4]
    where
        f32: AsPrimitive<T> + From<T::Output>,
    {
        [
            self.red.clamp(),
            self.green.clamp(),
            self.blue.clamp(),
            self.alpha.clamp(),
        ]
    }
}
