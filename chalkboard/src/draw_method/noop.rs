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
