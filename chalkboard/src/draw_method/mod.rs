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

use crate::{DrawOperation, Result};

mod noop;
pub use noop::NoopDrawer;

mod trapezoids;
pub use trapezoids::TrapezoidMethod;

/// The backing interface for drawing things.
pub trait DrawMethod {
    /// Get the inner `DrawMethod` backing this one.
    fn inner(&mut self) -> &mut dyn DrawMethod;

    /// Run a `DrawOperation`.
    fn draw(&mut self, op: &DrawOperation<'_>) -> Result<()>;

    /// Flush any pending data to the underlying surface.
    ///
    /// By default, this function is a no-op.
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<D: DrawMethod + ?Sized> DrawMethod for &mut D {
    fn inner(&mut self) -> &mut dyn DrawMethod {
        D::inner(self)
    }

    fn draw(&mut self, op: &DrawOperation<'_>) -> Result<()> {
        D::draw(self, op)
    }

    fn flush(&mut self) -> Result<()> {
        D::flush(self)
    }
}
