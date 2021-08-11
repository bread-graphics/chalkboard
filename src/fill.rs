// MIT/Apache2 License

use crate::{gradient::Gradient, Color};
use lyon_geom::Angle;

/// Defines how a particular space is filled.
#[derive(Debug, Clone, PartialEq)]
pub enum FillRule {
    SolidColor(Color),
    LinearGradient(Gradient<'static>, Angle<f32>),
    RadialGradient(Gradient<'static>),
    ConicalGradient(Gradient<'static>),
}
