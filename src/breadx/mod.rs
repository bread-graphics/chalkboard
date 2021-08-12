// MIT/Apache2 License

mod fallback;
#[cfg(feature = "xrender")]
mod xrender;

pub use fallback::*;
#[cfg(feature = "xrender")]
pub use xrender::*;

pub(crate) mod image;
