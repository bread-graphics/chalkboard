// BSL 1.0 License

use crate::{DrawOperation, Result};
use core::any::Any;
use genimage::{Format, Image};

/// The device is used to provide functionality to surfaces.
pub trait Device {
    /// The surface used to back this device.
    type Surface: Any;
    /// The image that the device's surfaces map onto.
    type Image: Image + Default;
    /// The return type of `supported_formats`.
    type SupportedFormats: IntoIterator<Item = Format>;

    /// Determine the formats supported by this device and its surface.
    fn supported_formats(&mut self, surface: &mut Self::Surface) -> Result<Self::SupportedFormats>;

    /// Initialize an image to a given size.
    fn initialize_image(
        &mut self,
        image: &mut Self::Image,
        width: usize,
        height: usize,
        format: Format,
    ) -> Result<()>;

    /// Map a surface's contents onto an image.
    fn map_surface_to_image(
        &mut self,
        surface: &mut Self::Surface,
        image: &mut Self::Image,
    ) -> Result<()>;

    /// Map an image's contents onto a surface.
    fn map_image_to_surface(
        &mut self,
        surface: &mut Self::Surface,
        image: &Self::Image,
    ) -> Result<()>;

    /// Run a draw operation on a surface.
    fn draw(&mut self, surface: &mut Self::Surface, op: DrawOperation) -> Result<()>;

    /// Cast a given surface to see if it is our surface.
    fn cast_our_surface<'a>(&mut self, surface: &'a mut dyn Any) -> Option<&'a mut Self::Surface> {
        surface.downcast_mut()
    }
}
