// MIT/Apache2 License

use std::fmt;

#[cfg(feature = "breadx")]
use breadx::BreadError;

/// Sum error type for chalkboard operations.
#[derive(Debug)]
pub enum Error {
    /// A static string message.
    StaticMsg(&'static str),
    /// A string message.
    Msg(String),
    /// A BreadX error occurred.
    #[cfg(feature = "breadx")]
    BreadX(BreadError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StaticMsg(s) => f.write_str(s),
            Self::Msg(s) => f.write_str(s),
            #[cfg(feature = "breadx")]
            Self::BreadX(bx) => fmt::Display::fmt(bx, f),
        }
    }
}

#[cfg(feature = "breadx")]
impl From<BreadError> for Error {
    #[inline]
    fn from(be: BreadError) -> Self {
        Self::BreadX(be)
    }
}

/// Convenience result type.
pub type Result<T = ()> = std::result::Result<T, Error>;
