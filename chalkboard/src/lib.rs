// MIT/Apache2 License

//#![forbid(unsafe_code)]

mod error;

pub mod fill;
pub mod gradient;
pub mod surface;

#[cfg(all(unix, feature = "breadx"))]
pub mod breadx;
#[cfg(all(windows, feature = "yaww"))]
pub mod yaww;

pub(crate) mod util;

pub use error::*;
pub use fill::*;
pub use gradient::*;
pub use surface::*;

#[doc(inline)]
pub use chalkboard_geometry::{
    Angle, BezierCurve, Color, GeometricArc, Intensity, Line, Path, PathSlice, Point, Rectangle,
};
