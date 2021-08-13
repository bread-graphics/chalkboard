// MIT/Apache2 License

use super::PixmapPicture;
use crate::ImageFormat;

/// Convert an image to an X11 `PixmapPicture`.
#[inline]
pub(crate) fn image_to_pixmap_picture<Dpy: Display + ?Sized>(
    display: &mut Dpy,
    target: Drawable,
    image_butes: &[u8],
    width: u32,
    height: u32,
    format: ImageFormat,
) -> crate::Result<PixmapPicture> {
    let window = Window::const_from_xid(target.xid);
    let geom_key = window.geometry(display)?;
    let attr_key = window.window_attributes(display)?;
    let GetGeometryReply { depth, .. } = display.resolve_request(geom_key)?;
    let GetWindowAttributesReply { visualid, .. } = display.resolve_request(attr_key);

    // create an appropriate image for both the main image and the alpha mask
    let visual = display
        .visual_id_to_visual(visualid)
        .ok_or(crate::Error::ImageNotAvailable)?;

    let quantum = match image_format {
        ImageFormat::Grayscale => 1,
        ImageFormat::Rgb | ImageFormat::Rgba => 4,
    };
    let heap_space: Box<[u8]> =
        unsafe { Box::new_zeroed_slice(quantum * (width * height) as usize).assume_init() };
    let mut base_image = breadx::Image::new(
        &display,
        Some(visual),
        depth,
        breadx::ImageFormat::ZPixmap,
        0,
        heap_space,
        width as usize,
        height as usize,
        (quantum * 8) as usize,
        None,
    )
    .ok_or(crate::Error::ImageNotAvailable)?;

    unimplemented!()
}
