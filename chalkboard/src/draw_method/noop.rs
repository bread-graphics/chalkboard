// BSL 1.0 License

use super::DrawMethod;
use crate::{DrawOperation, Result};

/// A `DrawMethod` that just returns errors or itself.
///
/// All other `DrawMethod`'s chains terminate in this link.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NoopDrawer;

impl DrawMethod for NoopDrawer {
    fn inner(&mut self) -> &mut dyn DrawMethod {
        self
    }

    fn draw(&mut self, _: &DrawOperation<'_>) -> Result<()> {
        Err(crate::Error::from_display(
            "Attempted to draw with a NoopDrawer",
        ))
    }
}
