// MIT/Apache2 License

use std::{fmt, num::NonZeroUsize};

#[cfg(all(unix, feature = "breadx"))]
use breadx::BreadError;

#[cfg(feature = "yaww")]
use yaww::Error as YawwError;

/// Sum error type for chalkboard operations.
#[derive(Debug)]
pub enum Error {
    /// A static string message.
    StaticMsg(&'static str),
    /// A string message.
    Msg(String),
    /// Attempted to run an unsupported operation.
    NotSupported(NSOpType),
    /// Attempted to initialize a display when we can't.
    NoInitializer,
    /// We do not have access to the given screen.
    NoScreen(usize),
    /// We do not know of the given window.
    NotOurWindow(NonZeroUsize),
    /// A BreadX error occurred.
    #[cfg(all(unix, feature = "breadx"))]
    BreadX(BreadError),
    /// A Yaww error occurred.
    #[cfg(all(windows, feature = "yaww"))]
    Yaww(YawwError),
}

/// An operation that is not supported.
#[derive(Debug, Copy, Clone)]
pub enum NSOpType {
    Gradients,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StaticMsg(s) => f.write_str(s),
            Self::Msg(s) => f.write_str(s),
            Self::NotSupported(nsop) => {
                write!(f, "Surface does not support feature \"{:?}\"", nsop)
            }
            Self::NoInitializer => f.write_str("Could not find initializer for current platform"),
            Self::NoScreen(i) => write!(f, "Screen #{} does not exist", i),
            Self::NotOurWindow(w) => write!(f, "Window of ID {:#010x} does not exist", w),
            #[cfg(all(unix, feature = "breadx"))]
            Self::BreadX(bx) => fmt::Display::fmt(bx, f),
            #[cfg(all(windows, feature = "yaww"))]
            Self::Yaww(y) => fmt::Display::fmt(y, f),
        }
    }
}

#[cfg(all(unix, feature = "breadx"))]
impl From<BreadError> for Error {
    #[inline]
    fn from(be: BreadError) -> Self {
        Self::BreadX(be)
    }
}

#[cfg(feature = "yaww")]
impl From<YawwError> for Error {
    #[inline]
    fn from(ye: YawwError) -> Self {
        Self::Yaww(ye)
    }
}

/// Convenience result type.
pub type Result<T = ()> = std::result::Result<T, Error>;
