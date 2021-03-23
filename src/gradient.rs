// MIT/Apache2 License

use crate::{color::Color, intensity::Intensity};
use std::{
    iter::FromIterator,
    slice::{Iter as SliceIter, IterMut as SliceIterMut},
};
use tinyvec::{TinyVec, TinyVecIterator};

const EXPECTED_CSTOPS: usize = 3;

/// A gradient of colors.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Gradient {
    // invariant: contains at least 1 element
    colors: TinyVec<[ColorStop; EXPECTED_CSTOPS]>,
}

impl Gradient {
    /// Create a new gradient without checking to see if it fulfills its conditions.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if the iterator contains zero elements, or if the color stops are not in order by
    /// their position.
    #[inline]
    pub unsafe fn new_unchecked<I: IntoIterator<Item = ColorStop>>(i: I) -> Self {
        Self {
            colors: TinyVec::from_iter(i),
        }
    }

    /// Creates a new gradient from an iterator. If the iterator is empty, or if the `position` field of each
    /// color stop are not in order, this function returns `None`.
    #[inline]
    pub fn new<I: IntoIterator<Item = ColorStop>>(mut i: I) -> Option<Self> {
        let colors: TinyVec<[ColorStop; EXPECTED_CSTOPS]> = TinyVec::from_iter(i);
        if colors.len() == 0 || {
            // todo: use is_sorted() once that's stabilized
            colors
                .iter()
                .try_fold(
                    ColorStop {
                        position: unsafe { Intensity::new_unchecked(0.0f32) },
                        color: Color::BLACK,
                    },
                    |min, me| {
                        if me.position <= min.position {
                            None
                        } else {
                            Some(*me)
                        }
                    },
                )
                .is_none()
        } {
            None
        } else {
            Some(Self { colors })
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
}

impl IntoIterator for Gradient {
    type Item = ColorStop;
    type IntoIter = TinyVecIterator<[ColorStop; EXPECTED_CSTOPS]>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.colors.into_iter()
    }
}

/// A color stop in a color gradient.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColorStop {
    pub color: Color,
    pub position: Intensity,
}
