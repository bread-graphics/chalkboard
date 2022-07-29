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

use super::{BoxDraw, Device, PatternAndOrigin};
use crate::{Result, SpecializedPattern};
use core::iter;
use genimage::{Image, Rgba};
use geometry::{Box2D, CompositeOperation, Size2D, Vector2D};

/// A device that is able to run compositing operations.
pub trait CompositeDraw: BoxDraw {
    /// Composite boxes from a pattern onto a surface.
    fn composite_boxes(
        &mut self,
        op: CompositeOperation,
        dst: &mut Self::Surface,
        src: PatternAndOrigin<'_, Self>,
        mask: PatternAndOrigin<'_, Self>,
        boxes: impl Iterator<Item = Box2D<f32>>,
    ) -> Result<()>;

    /// Composite a pattern onto a surface.
    ///
    /// By default, this calls `composite_boxes` with only a single
    /// box.
    fn composite(
        &mut self,
        op: CompositeOperation,
        dst: &mut Self::Surface,
        dst_mov: Vector2D<f32>,
        src: PatternAndOrigin<'_, Self>,
        mask: PatternAndOrigin<'_, Self>,
        size: Size2D<f32>,
    ) -> Result<()> {
        let pt = dst_mov.to_point();
        let next_pt = pt + size.to_vector();
        let b = Box2D::new(pt, next_pt);
        self.composite_boxes(op, dst, src, mask, iter::once(b))
    }
}
