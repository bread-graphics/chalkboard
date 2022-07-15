// BSL 1.0 License

use crate::{Result, DrawOperation};
use super::DrawMethod;

/// A `DrawMethod` that just returns errors or itself.
/// 
/// All other `DrawMethod`'s chains terminate in this link.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NoopDrawer;

impl DrawMethod for NoopDrawer {
    fn inner(&mut self) -> &mut dyn DrawMethod {
        self
    }

    fn draw(&mut self, _: DrawOperation) -> Result<()> {
        Err(crate::Error::unsupported())
    }
}