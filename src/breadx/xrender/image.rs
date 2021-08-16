// MIT/Apache2 License

use super::PixmapPicture;
use crate::ImageFormat;
use breadx::{
    auto::{
        render::{Color as XrColor, PictOp},
        xproto::{GetGeometryReply, GetWindowAttributesReply, Rectangle},
    },
    prelude::*,
    render::{Picture, RenderDisplay},
    Display, Drawable, Pixmap, Visualid, Window,
};
use std::collections::HashSet;

/// Convert an image to an X11 `PixmapPicture`.
#[inline]
pub(crate) fn image_to_pixmap_picture<Dpy: Display + ?Sized>(
    display: &mut RenderDisplay<Dpy>,
    target: Drawable,
    image_bytes: &[u8],
    width: u32,
    height: u32,
    format: ImageFormat,
) -> crate::Result<PixmapPicture> {
    // create the basic image
    let (image, visual, depth) = crate::breadx::image::breadx_image(
        display.inner_mut(),
        target,
        image_bytes,
        width,
        height,
        format,
    )?;

    // create the pixmap
    let pixmap = display
        .inner_mut()
        .create_pixmap_from_image(target, &image)?;

    // if this is not an alpha-channel, we're finished after we turn this into a pixmap-picture
    if format.has_alpha_component() {
        // TODO: this is not the most efficient way of doing this. if you know a better way, please open a PR!

        // create the pixmap-picture we're reading from
        let picture = into_pixmap_picture(display, pixmap, visual)?;

        // create an alpha mask
        let alpha_mask = display.create_pixmap(target, width as _, height as _, depth)?;

        // create a sigma mask
        let alpha_mask = into_pixmap_picture(display, alpha_mask, visual)?;

        // copy alpha bits to the alpha mask
        let mut x = 0u32;
        let mut y = 0u32;
        let mut pixels: Vec<(u8, u32, u32)> =
            crate::image::iterate_pixels(image_bytes, width, height, format)
                .map(|pixel| {
                    let this_x = x;
                    let this_y = y;
                    match x + 1 {
                        new_x if new_x >= width => {
                            x = 0;
                            y += 1;
                        }
                        new_x => {
                            x = new_x;
                        }
                    }

                    (format.alpha_component(pixel), this_x, this_y)
                })
                .collect();
        pixels.sort_unstable_by_key(|(key, _, _)| *key);

        let mut last_alpha = pixels.first().unwrap().0;
        let mut locations: Vec<(i16, i16)> = Vec::with_capacity(pixels.len());

        let (alpha, locations) = pixels.into_iter().try_fold(
            (last_alpha, locations),
            |(last_alpha, mut locations), (alpha, x, y)| {
                if alpha == last_alpha {
                    locations.push((x as i16, y as i16));
                    Result::<_, crate::Error>::Ok((last_alpha, locations))
                } else {
                    fill_pixels(display.inner_mut(), alpha_mask.picture, alpha, locations)?;
                    Ok((alpha, vec![(x as i16, y as i16)]))
                }
            },
        )?;

        fill_pixels(display.inner_mut(), alpha_mask.picture, alpha, locations)?;

        // now that we have created the alpha mask, run an "over" operation on a third pixmap
        let result = display.create_pixmap(target, width as _, height as _, depth)?;
        let result = into_pixmap_picture(display, result, visual)?;
        picture.picture.composite(
            display,
            PictOp::Over,
            alpha_mask.picture,
            result.picture,
            0,
            0,
            0,
            0,
            0,
            0,
            width as _,
            height as _,
        )?;

        picture.free(display)?;
        result.free(display)?;

        return Ok(result);
    }

    into_pixmap_picture(display, pixmap, visual)
}

#[inline]
fn into_pixmap_picture<Dpy: Display + ?Sized>(
    display: &mut RenderDisplay<Dpy>,
    pixmap: Pixmap,
    visual: Visualid,
) -> crate::Result<PixmapPicture> {
    let visual = display.inner().visual_id_to_visual(visual).unwrap();
    let format = display.find_visual_format(visual).unwrap();

    let picture = display.create_picture(pixmap, format, Default::default())?;

    Ok(PixmapPicture { pixmap, picture })
}

#[inline]
fn fill_pixels<Dpy: Display + ?Sized>(
    display: &mut Dpy,
    picture: Picture,
    alpha: u8,
    locations: Vec<(i16, i16)>,
) -> crate::Result {
    picture.fill_rectangles(
        display,
        PictOp::Src,
        XrColor {
            red: 0,
            green: 0,
            blue: 0,
            alpha: (alpha as u16) << 16,
        },
        locations
            .into_iter()
            .map(|(x, y)| Rectangle {
                x,
                y,
                width: 1,
                height: 1,
            })
            .collect::<Vec<Rectangle>>(),
    )?;
    Ok(())
}
