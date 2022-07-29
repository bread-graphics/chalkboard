// BSL 1.0 License

use core::mem;

use crate::{draw_method::NoopDrawer, Device, DrawMethod, DrawOperation, Result};

/// A context for drawing.
///
/// This is the centerpiece structure of the `chalkboard` library.
///
/// This is implemented as a wrapper around a `DrawMethod` with some
/// associated state.
pub struct Context<'a> {
    /// The current draw method.
    ///
    /// Realistically, this is never `None`. It is in an `Option` to
    /// make taking it out of the context and replacing it easier.
    /// Note that references are niched, so the runtime cost of this
    /// is likely to be very small.
    draw_method: Option<&'a mut dyn DrawMethod>,
}

impl<'a> Context<'a> {
    /// Create a new `Context` from the raw `DrawMethod`.
    pub fn new(draw_method: &'a mut dyn DrawMethod) -> Self {
        Context {
            draw_method: Some(draw_method),
        }
    }

    /// Get the current draw method.
    fn draw_method(&mut self) -> &mut dyn DrawMethod {
        self.draw_method.as_mut().expect("DrawMethod is None")
    }

    /// Run a `draw` operation.
    fn draw(&mut self, op: &DrawOperation<'_>) -> Result<()> {
        loop {
            // try to draw using the current draw method
            match self.draw_method().draw(op) {
                // if the drawing is unsupported, move on
                // to the next one
                Err(e) if e.is_unsupported() => {
                    let dm = self.draw_method.take().expect("DrawMethod is None");
                    let next = dm.inner();
                    self.draw_method = Some(next);
                }
                res => return res,
            }
        }
    }

    /// Flush any pending data to the underlying surface.
    pub fn flush(&mut self) -> Result<()> {
        self.draw_method().flush()
    }
}

impl<'a, D: DrawMethod> From<&'a mut D> for Context<'a> {
    fn from(draw_method: &'a mut D) -> Self {
        Self::new(draw_method)
    }
}
