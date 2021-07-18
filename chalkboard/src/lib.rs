// MIT/Apache2 License

#![forbid(unsafe_code)]

mod error;

pub mod color;
pub mod fill;
pub mod gradient;
pub mod surface;

#[cfg(all(unix, feature = "breadx"))]
pub mod breadx;
#[cfg(all(windows, feature = "yaww"))]
pub mod yaww;

pub(crate) mod util;

pub use color::*;
pub use error::*;
pub use fill::*;
pub use geometry::*;
pub use gradient::*;
pub use intensity::*;
pub use path::*;
pub use surface::*;

#[doc(inline)]
pub use chalkboard_geometry::{
    Angle, BezierCurve, GeometricArc, Intensity, Line, Path, Point, Rectangle,
};
