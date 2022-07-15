// BSL 1.0 License

//! Drawing primitives for usage in `chalkboard` and `chalkboard`-oriented applications.
//! 
//! These are mostly re-exports from [`euclid`] and [`lyon`], but there are certain new
//! primitives that are essential for rendering.
//! 
//! [`euclid`]: https://docs.rs/euclid/
//! [`lyon`]: https://docs.rs/lyon/


#![no_std]

extern crate alloc;

mod clip;
pub use clip::Clip;

mod region;
pub use region::Region;

mod trap;
pub use trap::Trapezoid;

#[doc(inline)]
pub use euclid::{
    Angle,
    default::{
        Box2D,
        Length,
        Point2D,
        Rect,
        Rotation2D,
        Size2D,
        Transform2D,
        Translation2D,
        Vector2D,
    }
};

#[doc(inline)]
pub use lyon_geom::{
    Arc,
    ArcFlags,
    Line,
    LineSegment,
    Triangle,
};

#[doc(inline)]
pub use lyon_path::{
    Path,
    PathBuffer,
    PathSlice,
    PathBufferSlice,
    Event as PathEvent,
    builder::PathBuilder,
};