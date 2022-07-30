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
