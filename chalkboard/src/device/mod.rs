// BSL 1.0 License

use crate::{DrawMethod, DrawOperation, Result, SpecializedPattern};
use core::any::Any;
use genimage::{Format, Image};

mod boxes;
mod composite;
mod image_map;
mod trapezoids;

pub use boxes::BoxDraw;
pub use composite::CompositeDraw;
use geometry::Vector2D;
pub use image_map::ImageMapDraw;
pub use trapezoids::TrapezoidDraw;

/// The device is used to provide functionality to surfaces.
pub trait Device {
    /// The surface used to back this device.
    type Surface: Any;

    /// Get a drawing method for a device and a corresponding surface.
    fn draw_method<R>(
        &mut self,
        surface: &mut Self::Surface,
        format: impl FnOnce(&mut dyn DrawMethod) -> Result<R>,
    ) -> Result<R>;

    /// Cast a given surface to see if it is our surface.
    fn cast_our_surface<'a>(
        &mut self,
        surface: &'a mut dyn Any,
    ) -> core::result::Result<&'a mut Self::Surface, &'a mut dyn Any> {
        if surface.is::<Self::Surface>() {
            // the compiler should be able to optimize this out
            Ok(surface.downcast_mut().unwrap())
        } else {
            Err(surface)
        }
    }
}

/// A `SpecializedPattern` with an offset specifiying which coordinate
/// in the pattern to start from.
///
/// This is useful for patterns that are not centered around the origin.
pub struct PatternAndOrigin<'surf, Dev: Device + ?Sized> {
    /// The pattern to draw.
    pub pattern: SpecializedPattern<'surf, Dev>,
    /// The origin point to start drawing from.
    pub origin: Vector2D<f32>,
}

impl<'surf, Dev: Device + ?Sized> From<SpecializedPattern<'surf, Dev>>
    for PatternAndOrigin<'surf, Dev>
{
    fn from(pattern: SpecializedPattern<'surf, Dev>) -> Self {
        PatternAndOrigin {
            pattern,
            origin: Vector2D::zero(),
        }
    }
}
