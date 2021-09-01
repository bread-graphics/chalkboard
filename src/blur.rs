// MIT/Apache2 License

/// Represents a blurring of drawing operations.
pub enum Blur {
    /// A gaussian blur.
    Gaussian { sigma: f32, radius: f32, alpha: f32 },
}
