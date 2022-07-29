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

use super::{Device, PatternAndOrigin};
use crate::{Result, SpecializedPattern};
use genimage::{Image, Rgba};
use geometry::{Box2D, Vector2D};

/// A device that can draw boxes of several types onto its surface.
pub trait BoxDraw: Device {
    /// Fill boxes with a solid color.
    fn solid_color_boxes(
        &mut self,
        surface: &mut Self::Surface,
        color: Rgba,
        boxes: impl Iterator<Item = Box2D<f32>>,
    ) -> Result<()>;

    /// Fill boxes using an image type.
    fn image_boxes(
        &mut self,
        surface: &mut Self::Surface,
        image: &impl Image,
        image_mov: Vector2D<f32>,
        boxes: impl Iterator<Item = Box2D<f32>>,
    ) -> Result<()>;

    /// Fill boxes by blitting from one surface to another.
    fn blit_boxes(
        &mut self,
        dst: &mut Self::Surface,
        src: &mut Self::Surface,
        src_mov: Vector2D<f32>,
        boxes: impl Iterator<Item = Box2D<f32>>,
    ) -> Result<()>;

    /// Fill the boxes from a specific pattern.
    fn fill_boxes(
        &mut self,
        dst: &mut Self::Surface,
        pattern: &mut PatternAndOrigin<'_, Self>,
        boxes: impl Iterator<Item = Box2D<f32>>,
    ) -> Result<()> {
        let PatternAndOrigin { pattern, origin } = pattern;

        match pattern {
            SpecializedPattern::SolidColor(clr) => self.solid_color_boxes(dst, *clr, boxes),
            SpecializedPattern::GeneralImage(img) => self.image_boxes(dst, img, *origin, boxes),
            SpecializedPattern::Surface(src) => self.blit_boxes(dst, *src, *origin, boxes),
        }
    }
}
