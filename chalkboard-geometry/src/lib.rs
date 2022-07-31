//               Copyright John Nunley, 2022.
// Distributed under the Boost Software License, Version 1.0.
//       (See accompanying file LICENSE or copy at
//         https://www.boost.org/LICENSE_1_0.txt)

//! Drawing primitives for usage in `chalkboard` and `chalkboard`-oriented applications.
//!
//! These are mostly re-exports from [`euclid`] and [`lyon`], but there are certain new
//! primitives that are essential for rendering.
//!
//! [`euclid`]: https://docs.rs/euclid/
//! [`lyon`]: https://docs.rs/lyon/

#![no_std]
#![forbid(unsafe_code, rust_2018_idioms)]

extern crate alloc;

mod clip;
pub use clip::Clip;

mod composite;
pub use composite::CompositeOperation;

mod region;
pub use region::Region;

mod slope;
pub use slope::Slope;

mod thrice;
pub(crate) use thrice::Thrice;

mod trap;
pub use trap::Trapezoid;

mod polygon;
pub use polygon::*;

mod util;
pub(crate) use util::approx_eq;

#[doc(inline)]
pub use euclid::{
    default::{
        Box2D, Length, Point2D, Rect, Rotation2D, Size2D, Transform2D, Translation2D, Vector2D,
    },
    Angle,
};

#[doc(inline)]
pub use lyon_geom::{Arc, ArcFlags, Line, LineSegment, Scalar, Triangle};

#[doc(inline)]
pub use lyon_path::{
    builder::PathBuilder, iterator::Flattened, Event as PathEvent, Path, PathBuffer,
    PathBufferSlice, PathSlice,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FillRule {
    Winding,
    EvenOdd,
}
