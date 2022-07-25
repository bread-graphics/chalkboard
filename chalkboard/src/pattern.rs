// BSL 1.0 License

use core::any::Any;
use genimage::GeneralImage;

/// A pattern acts as a source or a mask in composition operations.
pub enum Pattern<'surf> {
    /// Any pattern that may be represented as a `GeneralImage`.
    GeneralImage(GeneralImage<&'surf mut [u8]>),
    /// Use a surface as a source.
    ///
    /// This surface is assumed to belong to the `Device` that is
    /// currently being drawn on. If it does not, the behavior
    /// is undefined. If you would like to use a surface that does
    /// not belong to a `Device`, you should map it to an image and
    /// then use that image as a pattern.
    Surface(&'surf mut dyn Any),
}
