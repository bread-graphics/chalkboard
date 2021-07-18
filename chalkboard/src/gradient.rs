// MIT/Apache2 License

use crate::{color::Color, intensity::Intensity};
use std::{
    borrow::Cow,
    iter::FromIterator,
    slice::{Iter as SliceIter, IterMut as SliceIterMut},
};
use tinyvec::{TinyVec, TinyVecIterator};

const EXPECTED_CSTOPS: usize = 3;

/// A gradient of colors.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Gradient<'a> {
    // invariant: contains at least 1 element
    colors: Cow<'a, [ColorStop]>,
}

impl<'a> Gradient<'a> {
    /// Convert a `Cow<'_, [ColorStop]`>` into a gradient.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if the `Cow` contains no elements, or if the elements in the `Cow` are not sorted
    /// by their `position` field.
    #[inline]
    pub unsafe fn new_unchecked(colors: Cow<'_, [ColorStop]>) -> Gradient<'a> {
        Gradient { colors }
    }

    /// Creates a new gradient from an item that can be converted into a `Cow<'_, [ColorStop]>`. If the item is
    /// empty, this returns `None`. Note that the elements are sorted before the `Gradient` is returned.
    #[inline]
    pub fn new<Colors: Into<Cow<'a, [ColorStop]>>>(colors: Colors) -> Option<Gradient> {
        let mut colors = colors.into();
        if colors.is_empty() {
            None
        } else {
            colors.sort_by_key(|c| c.position);
            Some(Gradient { colors })
        }
    }

    /// Creates an iterator over these values.
    #[inline]
    pub fn iter(&self) -> SliceIter<'_, ColorStop> {
        self.colors.iter()
    }

    /// Creates a mutable iterator over these values.
    #[inline]
    pub fn iter_mut(&mut self) -> SliceIterMut<'_, ColorStop> {
        self.colors.iter_mut()
    }

    /// Get a slice reference to the color stop values.
    #[inline]
    pub fn as_slice(&self) -> &[ColorStop] {
        &*self.colors
    }

    /// Get the inner `Cow<'_, [ColorStop]>` out of the `Gradient`.
    #[inline]
    pub fn into_inner(self) -> Cow<'a, [ColorStop]> {
        self.colors
    }

    /// Convert this `Gradient<'_>` to a `Gradient<'static>`. This requires copying if the `Gradient` is
    /// borrowed.
    #[inline]
    pub fn into_owned(self) -> Gradient<'static> {
        match self.colors {
            Cow::Borrowed(colors) => Gradient {
                colors: Cow::Owned(colors.to_vec()),
            },
            Cow::Owned(colors) => Gradient {
                colors: Cow::Owned(colors),
            },
        }
    }
}

/// A color stop in a color gradient.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColorStop {
    pub color: Color,
    pub position: Intensity,
}
