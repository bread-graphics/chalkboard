// MIT/Apache2 License

use super::{Color, Gradient};
use lyon_geom::Angle;
use std::borrow::Cow;

/// The strategy to use when filling a space.
///
/// A shape can be filled either wth a solid color or using a gradient.
#[derive(Debug, Clone, PartialEq)]
pub enum FillRule<'a> {
    /// Solid color fill.
    Solid(Color),
    /// Linear gradient fill.
    Linear(Cow<'a, Gradient>, Angle<f32>),
    /// Radial gradient fill.
    Radial(Cow<'a, Gradient>),
    /// Conical gradient fill.
    Conical(Cow<'a, Gradient>),
}
