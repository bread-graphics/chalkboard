// MIT/Apache2 License

use crate::{color::Color, intensity::Intensity};
use tinyvec::TinyVec;

/// A gradient of colors.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Gradient {
    // invariant: contains at least 1 element
    colors: TinyVec<[ColorStop; 3]>,
}

/// A color stop in a color gradient.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColorStop {
    pub color: Color,
    pub intensity: Intensity,
}
