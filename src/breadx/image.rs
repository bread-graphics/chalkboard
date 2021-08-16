// MIT/Apache2 License

use crate::ImageFormat;
use breadx::{
    auto::xproto::{GetGeometryReply, GetWindowAttributesReply},
    prelude::*,
    Display, Drawable, Pixmap, Visualid, Window,
};
use std::num::NonZeroUsize;

#[inline]
pub(crate) fn breadx_image<Dpy: Display + ?Sized>(
    display: &mut Dpy,
    target: Drawable,
    image_bytes: &[u8],
    width: u32,
    height: u32,
    format: ImageFormat,
) -> crate::Result<(breadx::Image<Box<[u8]>>, Visualid, u8)> {
    // we need the target's depth and visual in order to construct the image
    let window = Window::const_from_xid(target.xid);
    let geom_key = window.geometry(display)?;
    let attr_key = window.window_attributes(display)?;

    let GetGeometryReply { depth, .. } = display.resolve_request(geom_key)?;
    let GetWindowAttributesReply {
        visual: visualid, ..
    } = display.resolve_request(attr_key)?;

    let visual = display
        .visual_id_to_visual(visualid)
        .ok_or(crate::Error::ImageNotAvailable)?;

    // allocate sufficient heap space for the image
    let quantum = match format {
        ImageFormat::Grayscale => 1usize,
        ImageFormat::Rgb | ImageFormat::Rgba => 4,
    };
    let heap_space: Box<[u8]> =
        unsafe { Box::new_zeroed_slice(quantum * (width * height) as usize).assume_init() };

    // construct the image
    let mut image = breadx::Image::new(
        &display,
        Some(visual),
        depth,
        breadx::ImageFormat::ZPixmap,
        0,
        heap_space,
        width as usize,
        height as usize,
        (quantum * 8) as u32,
        None,
    )
    .ok_or(crate::Error::ImageNotAvailable)?;

    // fill the image with pixels
    crate::image::iterate_pixels(image_bytes, width, height, format).fold(
        (0, 0),
        |(x, y), pixel| {
            let pixel = pixel
                .iter()
                .take(3)
                .enumerate()
                .fold(0, |pixel, (i, component)| {
                    pixel | ((*component as u32) << (i * 8))
                });
            image.set_pixel(x, y, pixel);

            // update x and y
            match x + 1 {
                x if x == (width as usize) => (0, y + 1),
                x => (x, y),
            }
        },
    );

    Ok((image, visualid, depth))
}

#[inline]
pub(crate) fn image_to_pixmap<Dpy: Display + ?Sized>(
    display: &mut Dpy,
    target: Drawable,
    image_bytes: &[u8],
    width: u32,
    height: u32,
    image_format: ImageFormat,
) -> crate::Result<Pixmap> {
    // get the image containing the image data
    let (image, _, _) = breadx_image(display, target, image_bytes, width, height, image_format)?;

    // create a pixmap and draw the image onto it
    let pixmap = display.create_pixmap_from_image(target, &image)?;

    Ok(pixmap)
}
