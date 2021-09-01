// MIT/Apache2 License

#![allow(unused_unsafe)]

//! `chalkboard` is a common drawing API that aims to abstract over system drawing APIs using a common,
//! capable system.
//!
//! In constructing a GUI framework, a common problem is abstracting over the drawing APIs of the various
//! inner systems in a way that the end consumer of the API can easily wrap their head around. `chalkboard`
//! aims to solve this problem.
//!
//! Most `chalkboard` systems are centered around a [`Context`], which is often an abstraction over the
//! connection to the server or central management object. The `Context` manages [`Image`]s, among other things.
//! Usually, the `Context` is accessible from the surface; logically, the `Context`'s functionality is a subset
//! of that of a `Surface`.
//!
//! The [`Surface`] is the whole point of the `chalkboard` API. Anything that can be drawn upon implements
//! `Surface`. Objects that implement `Surface` are usually wrappers around windows or off-screen images. In all
//! cases, `Surface`s are capable of the following operations:
//!
//! * Drawing geometric primitives using solid colors.
//! * Drawing the outlines of geometric primitives using solid colors.
//! * Drawing and filling paths using solid colors.
//! * Creating and destroying non-transparent [`Image`]s, as well as copying them into the image.
//! * Applying a "clipping region" where drawing will not occur, based off of an `Image` structure.
//!
//! In addition, the `Surface` can return a [`SurfaceFeatures`] object, which describes what additional things
//! the `Surface` is capable of. Calling these methods on a `Surface` without the enabled feature will result
//! in an error. Additional features include:
//!
//! * If the `gradients` field is `true`, `Surface` operations can be filled using linear, radial and conical
//!   gradients.
//! * If the `transparancy` field is `true`, the `Surface` can handle images containing transparancy. Otherwise,
//!   the alpha channel is simply ignored.
//! * If the `floats` field is `true`, floating point values passed into the `Surface` will not be rounded down
//!   to the nearest integer.
//! * If the `transforms` field is `true`, an arbitrary matrix transformation can be applied to drawing.
//! * If the `blurs` field is `true`, gaussian and binomial blurs can be applied
//!
//! It is considered good practice to check the `features()` method on a `Surface` before using any of
//! the above.
//!
//! # Geometry
//!
//! This crate is built upon the geometry primitives provided by [`lyon`] and [`euclid`]. Please see the
//! documentations for those crates for more information.
//!
//! [`lyon`]: https://crates.io/crates/lyon
//! [`euclid`]: https://crates.io/crates/euclid
//!
//! # Implementations
//!
//! `chalkboard` implements `Context` and `Surface` for several system drawing APIs, including:
//!
//! * [`breadx`] - Provides two separate APIs: one for drawing using the `xproto` default fallback graphics,
//!                and one for using the `render` graphics system, which is faster and much more capable.
//! * `yaww` - Provides an API for drawing using WinGDI.
//! * TODO: Direct2D, MacOSX and OpenGL
//!
//! [`breadx`]: https://crates.io/crates/breadx
//!
//! # Residuals and Shared Objects
//!
//! During operation of `chalkboard` `Surface`s, it becomes optimal to cache some objects. For instance, if a
//! brush object is created, it is more efficient to keep it around than to delete it, only for it to be
//! created again the next time the draw handler is called. In order to get around this, most `chalkboard`
//! objects return "residuals".
//!
//! The individual `Surface`s will have better documentation on this; however, as a rule of thumb, residuals
//! are created using the `into_residual()` method and consumed via the `from_residual()` or `free()` methods.
//! It is expected that the GUI framework that uses `chalkboard` will have a system for managing these
//! residuals.
//!
//! In addition, it is also required at points to keep track of data that will be stored between many
//! `Surface`s; for instance, a map containing information in regards to all of the extant `Image`s. For this
//! use case, many `Surface`s also required "shared objects". These "shared objects" are usually also kept
//! track of by the `Context` in real-world cases. Thus, `Surface`s may require these objects (or references
//! to them) to be passed in on creation. After creation, they are stored in the residual.

mod blur;
mod color;
mod context;
mod fill;
mod gradient;
mod image;
mod intensity;
mod surface;

pub(crate) mod path_utils;

pub use blur::*;
pub use color::*;
pub use context::*;
pub use fill::*;
pub use gradient::*;
pub use image::*;
pub use intensity::*;
pub use surface::*;

use lyon_geom::{Point, Vector};
use std::fmt;

/// A variety of things that can go wrong in `chalkboard` or the GUI framework that uses it.
#[derive(Debug)]
pub enum Error {
    /// The given operation is not supported.
    NotSupported(NotSupportedOp),
    /// The [`Image`] that was passed into a [`Surface`] does not support that `Surface`.
    NotOurImage(Image),
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotSupported(op) => write!(f, "Surface does not support operation: {:?}", op),
            Error::NotOurImage(image) => write!(
                f,
                "Image does not belong to surface: {:#010x}",
                image.into_raw()
            ),
        }
    }
}

/// Operations that may not be supported by all surfaces.
#[derive(Debug)]
pub enum NotSupportedOp {
    Transforms,
    Blurs,
    Gradients,
}

/// Result type for convenience.
pub type Result<T = ()> = std::result::Result<T, Error>;

/// An ellipse. Everyone knows what an ellipse is.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Ellipse {
    pub center: Point<f32>,
    pub radii: Vector<f32>,
}
