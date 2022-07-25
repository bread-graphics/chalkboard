// BSL 1.0 License

use crate::{Device, DrawMethod, Result};

/// A context for drawing.
///
/// This is the centerpiece structure of the `chalkboard` library.
///
/// This is implemented as a wrapper around a `DrawMethod` with some
/// associated state.
pub struct Context<'a> {
    draw_method: &'a mut dyn DrawMethod,
}

impl<'a> Context<'a> {
    /// Create a new `Context` from the raw `DrawMethod`.
    pub fn new(draw_method: &'a mut dyn DrawMethod) -> Self {
        Context { draw_method }
    }
}

impl<'a> From<&'a mut dyn DrawMethod> for Context<'a> {
    fn from(draw_method: &'a mut dyn DrawMethod) -> Self {
        Self::new(draw_method)
    }
}
