// BSL 1.0 License

use crate::{DrawOperation, Result};

mod noop;
pub use noop::NoopDrawer;

/// The backing interface for drawing things.
pub trait DrawMethod {
    /// Get the inner `DrawMethod` backing this one.
    fn inner(&mut self) -> &mut dyn DrawMethod;

    /// Run a `DrawOperation`.
    fn draw(&mut self, op: DrawOperation) -> Result<()>;
}
