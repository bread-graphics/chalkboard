// BSL 1.0 License

use super::Pattern;
use geometry::{Clip, CompositeOperation, PathBuffer};

/// An operation for drawing on a surface.
pub enum DrawOperation<'surf> {
    /// A straight-composite operation, mask and all.
    Mask { params: CompositeParameters<'surf> },
    /// Fill in the given paths.
    Fill {
        params: CompositeParameters<'surf>,
        paths: PathBuffer,
    },
    /// Outline the strokes of the given paths.
    Stroke {
        params: CompositeParameters<'surf>,
        paths: PathBuffer,
    },
}

/// Parameters for drawing on a surface.
pub struct CompositeParameters<'surf> {
    /// The operation combining the source and mask.
    operation: CompositeOperation,
    /// The clipping pattern, used to determine which
    /// areas, if any, to effect.
    clip: Clip,

    /// The source pattern.
    source: Pattern<'surf>,
    /// The mask pattern.
    mask: Pattern<'surf>,
}
