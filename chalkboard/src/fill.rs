// MIT/Apache2 License

use crate::{gradient::Gradient, Angle, Color};

/// Defines how a particular space is filled.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FillRule {
    SolidColor(Color),
    LinearGradient(Gradient<'static>, Angle),
    RadialGradient(Gradient<'static>),
    ConicalGradient(Gradient<'static>),
}
