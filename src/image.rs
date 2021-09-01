// MIT/Apache2 License

use std::num::NonZeroUsize;

/// Represents a server-side image structure.
///
/// In general, there are two types of images: those that exist on the client side, and those that exist on the
/// server side. Client-side images are ones that the client, or the program itself, has direct access to. These
/// usually consist of arrays of pixel data. Abstractions for these exist already, such as the [`ImageBuffer`]
/// structure.
///
/// [`ImageBuffer`]: https://docs.rs/image/*/image/struct.ImageBuffer.html
///
/// This struct aims to act as an abstraction over server-side images. The server, or what's actually doing the
/// GUI rendering, controls this image. This is usually just a pointer or ID that the server uses to identify
/// the image. Although it can't be modified without going through the server, it is often required to render
/// the image onto a window, and it is sometimes more efficient to move around.
///
/// The `Image` struct itself is just a thin wrapper around the [`NonZeroUsize`] structure, since it really is
/// either just an ID or a pointer underneath it all. Thus, there is no automatic management or cleanup of the
/// `Image`'s resources.
///
/// [`NonZeroUsize`]: https://doc.rust-lang.org/std/num/struct.NonZeroUsize.html
///
/// # Construction
///
/// `Image`s are crated via the `create_image` method on [`Context`] and [`Surface`]. Four components need to be
/// passed into the method:
///
/// * `image_bytes`, a `u8` slice consisting of the actual pixels the image is made up of. For instance, for an
///   RGB image, `image_bytes` would consist of sets of three bytes representing the red, green and blue
///   channels respectively. For users of `ImageBuffer`, this would be the result of the `container()` method.
/// * `width` and `height`, representing the size of the image being passed in.
/// * `format`, an instance of the [`ImageFormat`] enum that describes how the `image_bytes` slice is laid out.
///    See the definition of `ImageFormat` for information on available options.
///
/// If you are implementing your own `Surface` type, you can use the `from_raw` and `into_raw` methods to create
/// `Image`s. These are not recommended for end consumers.
///
/// # Cleanup
///
/// Once you are done using an `Image`, it is highly recommended to deallocate its resources using the
/// `destroy_image` method. Failing to do so can take up memory on the server side and can even lead to an OOM
/// condition in the worst case.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Image {
    inner: NonZeroUsize,
}

impl Image {
    /// Create a new `Image` from a `NonZeroUsize` representing a server-side image.
    #[inline]
    pub fn from_raw(raw: NonZeroUsize) -> Image {
        Image { inner: raw }
    }

    /// Get the `NonZeroUsize` backing this `Image`.
    #[inline]
    pub fn into_raw(self) -> NonZeroUsize {
        self.inner
    }
}

/// The format that an `Image` can have.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImageFormat {
    /// Grayscale format. Each element of the byte slice is a pixel representing how bright it is.
    Grayscale,
    /// Every three elements of the byte slice is an array of channels consisting of red, green and blue.
    Rgb,
    /// Every four elements of the byte slice is an array of channels consisting of red, green, blue and alpha.
    Rgba,
}

impl ImageFormat {
    /// Does this `ImageFormat` require transparency to be supported?
    #[inline]
    pub fn is_transparent(self) -> bool {
        matches!(self, ImageFormat::Rgba)
    }

    /// If divided into chunks, which component is the transparent one?
    ///
    /// # Panics
    ///
    /// Panics if this is not a transparent format.
    #[inline]
    pub fn transparent_component(self) -> usize {
        match self {
            ImageFormat::Rgba => 3,
            _ => panic!("Not a transparent format: {:?}", self),
        }
    }

    /// Get the quantum (i.e. bytes per pixel) for this image.
    #[inline]
    pub fn quantum(self) -> usize {
        match self {
            ImageFormat::Grayscale => 1,
            ImageFormat::Rgb => 3,
            ImageFormat::Rgba => 4,
        }
    }

    /// Divide a given image into chunks representing arrays.
    #[inline]
    pub fn into_chunks(self, bytes: &[u8], width: u32, height: u32) -> impl Iterator<Item = &[u8]> {
        bytes.chunks(self.quantum()).take((width * height) as usize)
    }
}
