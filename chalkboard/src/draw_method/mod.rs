// BSL 1.0 License

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
