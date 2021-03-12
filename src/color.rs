// MIT/Apache2 License

use ordered_float::NotNan;

/// A four-element color.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Color {
    r: NotNan<f32>,
    g: NotNan<f32>,
    b: NotNan<f32>,
    a: NotNan<f32>,
}

impl Color {
    /// Create a new color.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if any of the elements are NaN.
    #[inline]
    pub const unsafe fn new_unchecked(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: NotNan::unchecked_new(r),
            g: NotNan::unchecked_new(g),
            b: NotNan::unchecked_new(b),
            a: NotNan::unchecked_new(a),
        }
    }

    /// Creates a new color. This function returns `None` if any of the elements are NaN.
    #[inline]
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Option<Self> {
        Some(Self {
            r: NotNan::new(r).ok()?,
            g: NotNan::new(g).ok()?,
            b: NotNan::new(b).ok()?,
            a: NotNan::new(a).ok()?,
        })
    }

    /// Gets the red element.
    #[inline]
    pub fn red(self) -> f32 {
        self.r.into_inner()
    }

    /// Gets the green element.
    #[inline]
    pub fn green(self) -> f32 {
        self.g.into_inner()
    }

    /// Gets the blue element.
    #[inline]
    pub fn blue(self) -> f32 {
        self.b.into_inner()
    }

    /// Gets the alpha element.
    #[inline]
    pub fn alpha(self) -> f32 {
        self.a.into_inner()
    }
}
