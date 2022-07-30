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

use super::Pattern;
use geometry::{Clip, CompositeOperation, PathBuffer};

/// An operation for drawing on a surface.
pub enum DrawOperation<'surf> {
    /// A straight-composite operation, mask and all.
    Mask { params: CompositeParameters<'surf> },
    /// Fill in the given paths.
    Fill {
        params: CompositeParameters<'surf>,
        paths: PathBuffer,
    },
    /// Outline the strokes of the given paths.
    Stroke {
        params: CompositeParameters<'surf>,
        paths: PathBuffer,
    },
}

/// Parameters for drawing on a surface.
pub struct CompositeParameters<'surf> {
    /// The operation combining the source and mask.
    operation: CompositeOperation,
    /// The clipping pattern, used to determine which
    /// areas, if any, to effect.
    clip: Clip,

    /// The source pattern.
    source: Pattern<'surf>,
    /// The mask pattern.
    mask: Pattern<'surf>,
}
