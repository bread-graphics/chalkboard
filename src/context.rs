// MIT/Apache2 License

use crate::{Image, ImageFormat};

/// The backing context behind a [`Surface`].
///
/// In real-world applications, `Surface`s rarely act alone. There is some kind of central node tying all
/// of them together. For X11 and Wayland protocols, there is a connection to the server over which all
/// interactions actually take place. For Appkit, there is an "application" object that holds the logic
/// necessary for interaction. For Win32, there is some kind of client-side logic keeping track of all
/// of the pointers. The `Context` trait aims to be an abstraction over these.
///
/// For our purposes, the `Context` exists mostly for creating and destroying `Image`s. In a GUI
/// framework, `Context`s are capable of much more; however, this is not a GUI framework.
pub trait Context {
    /// Creates a new `Image`.
    ///
    /// The `Image` is created in the `Context`'s format using client-side data. The `image_bytes` slice
    /// is expected to contain bytes in accordance with the supplied `ImageFormat`. The `width` and `height`
    /// specify the dimensions of the image.
    fn create_image(
        &mut self,
        image_bytes: &[u8],
        width: u32,
        height: u32,
        format: ImageFormat,
    ) -> crate::Result<Image>;

    /// Destroys an `Image`.
    ///
    /// This takes an `Image` and deallocates its memory. While this is not strictly necessary as memory
    /// leaks are a perfectly valid operation, it is not recommended to not clean up your images unless
    /// your program is already exiting.
    fn destroy_image(&mut self, image: Image) -> crate::Result;
}

impl<C: Context + ?Sized> Context for &mut C {
    #[inline]
    fn create_image(
        &mut self,
        image_bytes: &[u8],
        width: u32,
        height: u32,
        format: ImageFormat,
    ) -> crate::Result<Image> {
        (**self).create_image(image_bytes, width, height, format)
    }
    #[inline]
    fn destroy_image(&mut self, image: Image) -> crate::Result {
        (**self).destroy_image(image)
    }
}
