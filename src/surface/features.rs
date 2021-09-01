// MIT/Apache2 License

/// Features that are enabled on the [`Surface`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SurfaceFeatures {
    /// Fill rules for this `Surface` can use gradients as fills.
    pub gradients: bool,
    /// Transparency is accounted for in colors, including images.
    pub transparency: bool,
    /// Float values are not rounded down, allowing for sub-pixel drawing.
    pub floats: bool,
    /// The methods `set_transform()` and `remove_transform()` are available.
    pub transforms: bool,
    /// The methods `set_blur()` and `remove_blur()` are available.
    pub blurs: bool,
}
