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

use alloc::string::{String, ToString};
use core::fmt;

pub struct Error {
    kind: Kind,
}

enum Kind {
    Unsupported,
    InvalidInput(InvalidInput),
    Display(String),
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum InvalidInput {}

impl Error {
    /// Create a new error from an error-like type.
    pub fn from_display(f: impl fmt::Display) -> Self {
        Error {
            kind: Kind::Display(f.to_string()),
        }
    }

    /// Create a new unsupported error.
    pub fn unsupported() -> Self {
        Error {
            kind: Kind::Unsupported,
        }
    }

    /// Is this error an unsupported error?
    pub fn is_unsupported(&self) -> bool {
        matches!(self.kind, Kind::Unsupported)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct KindFmt<'a>(&'a Kind);

        impl<'a> fmt::Debug for KindFmt<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self.0 {
                    Kind::Unsupported => f.write_str("Unsupported"),
                    Kind::InvalidInput(i) => fmt::Debug::fmt(i, f),
                    Kind::Display(s) => write!(f, r#""{}""#, s),
                }
            }
        }

        f.debug_tuple("Error").field(&KindFmt(&self.kind)).finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            Kind::Unsupported => f.write_str("Attempted to run an unsupported operation"),
            Kind::InvalidInput(i) => match i {},
            Kind::Display(ref msg) => f.write_str(msg),
        }
    }
}

pub type Result<T = ()> = core::result::Result<T, Error>;
