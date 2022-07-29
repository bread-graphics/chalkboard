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

use super::CompositeDraw;
use crate::{Result, SpecializedPattern};
use geometry::{CompositeOperation, Trapezoid, Vector2D};

/// A device that can composite trapezoids onto its surface.
pub trait TrapezoidDraw: CompositeDraw {
    /// Composite trapezoids from a pattern onto a surface.
    fn composite_trapezoids(
        &mut self,
        op: CompositeOperation,
        dst: &mut Self::Surface,
        src: SpecializedPattern<'_, Self>,
        src_mov: Vector2D<f32>,
        trapezoids: impl Iterator<Item = Trapezoid<f32>>,
    ) -> Result<()>;
}
