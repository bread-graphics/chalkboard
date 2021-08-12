// MIT/Apache2 License

use std::num::NonZeroUsize;

/// An image on the server side.
///
/// Most APIs that `chalkboard` interacts with have two forms of images: on the client side, most often
/// represented by an array of bytes containing the image's pixels, and on the server side, most often
/// represented via a pointer or key that the server recognizes as the image.
///
/// Images on the client side can be effectively represented via existing structures, such as the `image::Image`
/// structure. This structure represents images on the server side.
///
/// In most cases, these are more efficient to deal with than standard client-side images. These can be created
/// via the `Surface::submit_image` method, and dropped via the `Surface::destroy_image` method.
///
/// This is represented using a `NonZeroUsize` structure, as most images are either numerical keys or pointers,
/// both of which can be represented as a non-zero number.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Image {
    inner: NonZeroUsize,
}

impl Image {
    #[inline]
    pub fn from_raw(inner: NonZeroUsize) -> Image {
        Image { inner }
    }

    #[inline]
    pub fn into_raw(self) -> NonZeroUsize {
        self.inner
    }
}

/// The supported formats than a client-side image can have.
///
/// See documentation on variants for information on the format that the bytes are expected to take.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImageFormat {
    /// Each byte represents the intensity of a pixel. The bytes are expected to be a list of pixels, with one
    /// byte per pixel.
    Grayscale,
    /// Every group of three bytes represents the intensity of the red, blue and green on a pixel. Three bytes
    /// per pixel.
    Rgb,
    /// Every group of four bytes represents the intensity of the red, blue, green and alpha components. Four
    /// bytes per pixel.
    Rgba,
}

impl ImageFormat {
    #[inline]
    pub fn has_alpha_component(self) -> bool {
        matches!(self, ImageFormat::Rgba)
    }

    #[inline]
    pub fn alpha_component(self, pixel: &[u8]) -> u8 {
        match self {
            ImageFormat::Rgba => pixel[3],
            _ => panic!("Invalid format"),
        }
    }
}

/// Create an iterator over a set of pixels from a set of bytes.
#[inline]
pub(crate) fn iterate_pixels(
    bytes: &[u8],
    width: u32,
    height: u32,
    format: ImageFormat,
) -> impl Iterator<Item = &[u8]> {
    let chunks = match format {
        ImageFormat::Grayscale => bytes.chunks(1),
        ImageFormat::Rgb => bytes.chunks(3),
        ImageFormat::Rgba => bytes.chunks(4),
    };

    chunks.take((width * height) as usize)
}
