// MIT/Apache2 License

use super::{Color, Intensity};
use std::{
    borrow::ToOwned,
    iter::FromIterator,
    mem,
    slice::{Iter, IterMut},
};

/// A gradient, representing a series of [`ColorStop`]s.
///
/// `Gradient`s can be held via either a reference or a `Box`. They cannot be owned by default.
///
/// `Gradient`s must have at least one element. Unsafe code can rely on this invariant.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Gradient {
    stops: [ColorStop],
}

impl Gradient {
    /// Create a new reference `Gradient` from a slice of [`ColorStop`]s without checking for emptiness.
    ///
    /// # Safety
    ///
    /// If the slice is empty, behavior is undefined.
    #[inline]
    pub unsafe fn from_slice_unchecked<'a>(sl: &'a [ColorStop]) -> &'a Gradient {
        // SAFETY: &'a [ColorStop] has the same layout as &'a Gradient
        unsafe { mem::transmute::<_, &'a Gradient>(sl) }
    }

    /// Create a new mutable reference `Gradient` from a slice of [`ColorStop`]s without checking for
    /// emptiness.
    ///
    /// # Safety
    ///
    /// If the slice is empty, behavior is undefined.
    #[inline]
    pub unsafe fn from_slice_mut_unchecked<'a>(sl: &'a mut [ColorStop]) -> &'a mut Gradient {
        // SAFETY: &'a mut [ColorStop] has the same layout at &'a mut Gradient
        unsafe { mem::transmute::<_, &'a mut Gradient>(sl) }
    }

    /// Create a new reference to a `Gradient` from a slice of [`ColorStop`]s.
    ///
    /// If the slice is empty, this function returns `None`.
    #[inline]
    pub fn from_slice<'a>(sl: &'a [ColorStop]) -> Option<&'a Gradient> {
        if sl.is_empty() {
            None
        } else {
            // SAFETY: invariant is fulfilled
            Some(unsafe { Gradient::from_slice_unchecked(sl) })
        }
    }

    /// Create a new mutable reference to a `Gradient` from a slice of [`ColorStop`]s.
    ///
    /// If the slice is empty, this function returns `None`.
    #[inline]
    pub fn from_slice_mut<'a>(sl: &'a mut [ColorStop]) -> Option<&'a mut Gradient> {
        if sl.is_empty() {
            None
        } else {
            // SAFETY: invariant is fulfilled
            Some(unsafe { Gradient::from_slice_mut_unchecked(sl) })
        }
    }

    /// Creates a new `Gradient` from a boxed slice of [`ColorStop`]s without checking for emptiness.
    ///
    /// # Safety
    ///
    /// If the slice is empty, behavior is undefined.
    #[inline]
    pub unsafe fn from_boxed_slice_unchecked(sl: Box<[ColorStop]>) -> Box<Gradient> {
        // SAFETY: same as above
        unsafe { Box::from_raw(mem::transmute::<_, *mut Gradient>(Box::into_raw(sl))) }
    }

    /// Creates a new `Gradient` from a boxed slice of [`ColorStop`]s.
    ///
    /// If the slice is empty, this function returns `None`.
    #[inline]
    pub fn from_boxed_slice(sl: Box<[ColorStop]>) -> Option<Box<Gradient>> {
        if sl.is_empty() {
            None
        } else {
            // SAFETY: invariant is fulfilled
            Some(unsafe { Gradient::from_boxed_slice_unchecked(sl) })
        }
    }

    /// Get the slice backing this `Gradient`.
    #[inline]
    pub fn as_slice<'a>(&'a self) -> &'a [ColorStop] {
        // SAFETY: layout is the same
        unsafe { mem::transmute::<_, &'a [ColorStop]>(self) }
    }

    /// Get the mutable slice backing this `Gradient`.
    #[inline]
    pub fn as_slice_mut<'a>(&'a mut self) -> &'a mut [ColorStop] {
        // SAFETY: layout is the same
        unsafe { mem::transmute::<_, &'a mut [ColorStop]>(self) }
    }

    /// Convert this `Gradient` back into a boxed slice.
    #[inline]
    pub fn into_boxed_slice(self: Box<Self>) -> Box<[ColorStop]> {
        // SAFETY: layout is the same
        unsafe { Box::from_raw(mem::transmute::<_, *mut [ColorStop]>(Box::into_raw(self))) }
    }
}

impl ToOwned for Gradient {
    type Owned = Box<Gradient>;

    #[inline]
    fn to_owned(&self) -> Box<Gradient> {
        unsafe { Gradient::from_boxed_slice_unchecked(self.as_slice().into()) }
    }
}

impl<'a> IntoIterator for &'a Gradient {
    type Item = &'a ColorStop;
    type IntoIter = Iter<'a, ColorStop>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.stops.iter()
    }
}

impl<'a> IntoIterator for &'a mut Gradient {
    type Item = &'a mut ColorStop;
    type IntoIter = IterMut<'a, ColorStop>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.stops.iter_mut()
    }
}

impl FromIterator<ColorStop> for Box<Gradient> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = ColorStop>>(i: I) -> Box<Gradient> {
        Gradient::from_boxed_slice(i.into_iter().collect()).expect("Iterator was empty")
    }
}

/// A color stop.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ColorStop {
    /// The color of this color stop.
    pub color: Color,
    /// Where the color stop is located in the gradient.
    pub location: Intensity,
}
