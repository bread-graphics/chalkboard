// MIT/Apache2 License

use crate::intensity::Intensity;

/// A four-element color.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Color {
    r: Intensity,
    g: Intensity,
    b: Intensity,
    a: Intensity,
}

impl Color {
    pub const WHITE: Color = unsafe { Color::new_unchecked(1.0, 1.0, 1.0, 1.0) };
    pub const BLACK: Color = unsafe { Color::new_unchecked(0.0, 0.0, 0.0, 1.0) };

    /// Create a new color.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if any of the elements are NaN.
    #[inline]
    pub const unsafe fn new_unchecked(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: Intensity::new_unchecked(r),
            g: Intensity::new_unchecked(g),
            b: Intensity::new_unchecked(b),
            a: Intensity::new_unchecked(a),
        }
    }

    /// Creates a new color. This function returns `None` if any of the elements are NaN.
    #[inline]
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Option<Self> {
        Some(Self {
            r: Intensity::new(r)?,
            g: Intensity::new(g)?,
            b: Intensity::new(b)?,
            a: Intensity::new(a)?,
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

    /// Clamp to u8's.
    #[inline]
    pub fn clamp_u8(self) -> (u8, u8, u8, u8) {
        let r = self.r.clamp_u8();
        let g = self.g.clamp_u8();
        let b = self.b.clamp_u8();
        let a = self.a.clamp_u8();
        (r, g, b, a)
    }

    /// Clamp to u16's.
    #[inline]
    pub fn clamp_u16(self) -> (u16, u16, u16, u16) {
        let r: u16 = self.r.clamp_u16();
        let g: u16 = self.g.clamp_u16();
        let b: u16 = self.b.clamp_u16();
        let a: u16 = self.a.clamp_u16();
        (r, g, b, a)
    }
}
