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

use super::{DrawMethod, NoopDrawer};
use crate::{device::TrapezoidDraw, Device, DrawOperation, Result};

/// A `DrawMethod` that draws by rendering paths as trapezoids and then
/// painting them to the underlying surface.
pub struct TrapezoidMethod<'surf, Dev: Device + ?Sized> {
    // device/surface to draw onto
    device: &'surf mut Dev,
    surface: &'surf mut Dev::Surface,

    // underlying NoopDrawer for when we're compromised
    noop: NoopDrawer,
}

impl<'surf, Dev: TrapezoidDraw + ?Sized> TrapezoidMethod<'surf, Dev> {
    /// Create a new `TrapezoidMethod` for the given device and surface.
    pub fn new(device: &'surf mut Dev, surface: &'surf mut Dev::Surface) -> Self {
        TrapezoidMethod {
            device,
            surface,
            noop: NoopDrawer,
        }
    }

    /// Get a reference to the underlying device.
    pub fn device(&self) -> &Dev {
        self.device
    }

    /// Get a mutable reference to the underlying device.
    pub fn device_mut(&mut self) -> &mut Dev {
        self.device
    }

    /// Get a reference to the underlying surface.
    pub fn surface(&self) -> &Dev::Surface {
        self.surface
    }

    /// Get a mutable reference to the underlying surface.
    pub fn surface_mut(&mut self) -> &mut Dev::Surface {
        self.surface
    }

    /// Get a tuple of references to the device and surface.
    pub fn device_and_surface(&self) -> (&Dev, &Dev::Surface) {
        (self.device, self.surface)
    }

    /// Get a tuple of mutable references to the device and surface.
    pub fn device_and_surface_mut(&mut self) -> (&mut Dev, &mut Dev::Surface) {
        (self.device, self.surface)
    }

    /// Convert this `DrawMethod` into the underlying
    /// device and surface.
    pub fn into_device_and_surface(self) -> (&'surf mut Dev, &'surf mut Dev::Surface) {
        let Self {
            device, surface, ..
        } = self;
        (device, surface)
    }
}

impl<'surf, Dev: TrapezoidDraw + ?Sized> DrawMethod for TrapezoidMethod<'surf, Dev> {
    fn inner(&mut self) -> &mut dyn DrawMethod {
        &mut self.noop
    }

    fn draw(&mut self, op: &DrawOperation<'_>) -> Result<()> {
        todo!()
    }
}
