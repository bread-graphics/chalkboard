// MIT/Apache2 License

use crate::{color::Color, geometry::Angle, gradient::Gradient};

/// Defines how a particular space is filled.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FillRule {
    SolidColor(Color),
    LinearGradient(Gradient<'static>, Angle),
    RadialGradient(Gradient<'static>),
    ConicalGradient(Gradient<'static>),
}
