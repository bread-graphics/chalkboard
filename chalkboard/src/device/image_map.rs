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

use super::Device;
use crate::Result;
use genimage::Image;

/// A device that can be mapped to a specific image type.
pub trait ImageMapDraw: Device {
    /// The image type that this device can be mapped onto.
    type Image: Image;

    /// Map a surface onto an image.
    fn map_surface_to_image(&mut self, surface: &mut Self::Surface) -> Result<Self::Image>;

    /// Map an image onto a surface.
    fn map_image_to_surface(
        &mut self,
        image: &impl Image,
        surface: &mut Self::Surface,
    ) -> Result<()>;
}
