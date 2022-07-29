// This file is part of chalkboard.
//
// chalkboard is free software: you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation,
// either version 3 of the License, or (at your option)
// any later version.
//
// chalkboard is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty
// of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General
// Public License along with chalkboard. If not, see
// <https://www.gnu.org/licenses/>.

use crate::Device;
use core::{any::Any, result::Result};
use genimage::{GeneralImage, Rgba};

/// A pattern acts as a source or a mask in composition operations.
pub enum Pattern<'surf> {
    /// A solid color.
    SolidColor(Rgba),
    /// Any pattern that may be represented as a `GeneralImage`.
    GeneralImage(GeneralImage<&'surf mut [u8]>),
    /// Use a surface as a source.
    ///
    /// This surface is assumed to belong to the `Device` that is
    /// currently being drawn on. If it does not, the behavior
    /// is undefined. If you would like to use a surface that does
    /// not belong to a `Device`, you should map it to an image and
    /// then use that image as a pattern.
    Surface(&'surf mut dyn Any),
}

/// A pattern specialized for a certain `Device`.
pub enum SpecializedPattern<'surf, Dev: Device + ?Sized> {
    /// A solid color.
    SolidColor(Rgba),
    /// Any pattern that may be represented as a `GeneralImage`.
    GeneralImage(GeneralImage<&'surf mut [u8]>),
    /// Use a surface as a source.
    Surface(&'surf mut Dev::Surface),
}

impl<'surf> Pattern<'surf> {
    pub fn specialize<D: Device + ?Sized>(
        self,
        device: &mut D,
    ) -> Result<SpecializedPattern<'surf, D>, Pattern<'surf>> {
        Ok(match self {
            Pattern::SolidColor(color) => SpecializedPattern::SolidColor(color),
            Pattern::GeneralImage(image) => SpecializedPattern::GeneralImage(image),
            Pattern::Surface(surface) => match device.cast_our_surface(surface) {
                Ok(surf) => SpecializedPattern::Surface(surf),
                Err(surface) => return Err(Pattern::Surface(surface)),
            },
        })
    }
}
