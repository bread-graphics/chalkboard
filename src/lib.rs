// MIT/Apache2 License

//#![forbid(unsafe_code)]

mod error;

pub mod color;
pub mod fill;
pub mod gradient;
pub mod intensity;
pub mod surface;

mod ellipse;
mod path;

#[cfg(all(unix, feature = "breadx"))]
pub mod breadx;
#[cfg(all(windows, feature = "yaww"))]
pub mod yaww;

pub(crate) mod util;

pub use color::*;
pub use ellipse::*;
pub use error::*;
pub use fill::*;
pub use gradient::*;
pub use intensity::*;
pub use surface::*;

pub(crate) use path::*;
