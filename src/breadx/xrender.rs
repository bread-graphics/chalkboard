// MIT/Apache2 License

use crate::{color::Color, fill::FillRule, gradient::Gradient, util::shouldnt_drop};
use breadx::{
    auto::render::{Color, Picture},
    display::{Connection, DisplayLike},
    render::{RenderDisplay, StandardFormat},
    Drawable, Pixmap,
};
use std::{
    collections::hash_map::HashMap,
    mem::{self, ManuallyDrop},
};
use tinyvec::TinyVec;

#[cfg(feature = "async")]
use breadx::display::AsyncConnection;

/// XRender-based BreadX surface. This is preferred over the XProto fallback, but it is generally preferred to
/// use GL rendering on systems that support it.
#[derive(Debug)]
pub struct RenderBreadxSurface<'dpy, Dpy> {
    // display
    display: &'dpy mut RenderDisplay<Dpy>,
    old_checked: bool,

    // drawable to create pixmaps on
    parent: Drawable,

    // target
    target: Picture,
    width: u16,
    height: u16,

    // we draw shapes onto this picture to use as a mask
    mask: PixmapPicture,

    // brushes associated with fill rules
    brushes: Option<HashMap<FillRule, PixmapPicture>>,

    // wait until the next garbage collection
    next_gc: usize,

    // emergency drop mechanism, if free() isnt called
    dropper: fn(&mut RenderBreadxSurface<'dpy, Dpy>),
}

impl<'dpy, Dpy> Drop for RenderBreadxSurface<'dpy, Dpy> {
    #[inline]
    fn drop(&mut self) {
        (self.dropper)(self)
    }
}

/// Residual from the RenderBreadxSurface, used to save space.
#[derive(Debug)]
pub struct RenderResidual {
    mask: PixmapPicture,
    brushes: Option<HashMap<FillRule, PixmapPicture>>,
    next_gc: usize,
    width: u16,
    height: u16,
}

impl RenderResidual {
    #[inline]
    pub fn free<Conn: Connection>(mut self, display: &mut Display<Conn>) -> crate::Result {
        self.mask.free(display)?;
        self.brushes
            .values()
            .try_for_each(|val| val.free(display))?;
        self.brushes.take();
        mem::forget(self);
        Ok(())
    }

    #[cfg(feature = "async")]
    #[inline]
    pub async fn free_async<Conn: AsyncConnection + Send>(
        mut self,
        display: &mut Display<Conn>,
    ) -> crate::Result {
        self.mask.free_async(display).await?;
        for val in self.brushes.values() {
            val.free_async(display).await?;
        }
        self.brushes.take();
        mem::forget(self);
        Ok(())
    }
}

impl Drop for RenderResidual {
    #[inline]
    fn drop(&mut self) {
        shouldnt_drop("RenderResidual")
    }
}

/// Combines a pixmap and a picture in one.
#[derive(Debug, Copy, Clone)]
struct PixmapPicture {
    pixmap: Pixmap,
    picture: Picture,

    // usage statistics since the last garbage collection
    usage: usize,
}

impl PixmapPicture {
    #[inline]
    fn free<Conn: Connection>(self, display: &mut Display<Conn>) -> crate::Result {
        self.picture.free(display)?;
        self.pixmap.free(display)?;
        Ok(())
    }

    #[cfg(feature = "async")]
    #[inline]
    async fn free_async<Conn: AsyncConnection + Send>(
        self,
        display: &mut Display<Conn>,
    ) -> crate::Result {
        self.picture.free_async(display).await?;
        self.pixmap.free_async(display).await?;
        Ok(())
    }

    #[inline]
    fn new<Dpy: DisplayLike>(
        display: &mut RenderDisplay<Dpy>,
        width: u16,
        height: u16,
        parent: Drawable,
        repeat: bool,
    ) -> crate::Result<PixmapPicture>
    where
        Dpy::Connection: Connection,
    {
        let pixmap = display
            .display_mut()
            .create_pixmap(parent, width, height, 8)?;
        let format = display
            .find_standard_format(StandardFormat::A8)
            .expect("A8 format not available");
        Ok(PixmapPicture {
            pixmap,
            picture: display.create_picture(parent, format, Default::default()),
            usage: 0,
        })
    }
}

impl<'dpy, Dpy> RenderBreadxSurface<'dpy, Dpy> {
    /// Convert this RenderBreadxSurface into the residual.
    #[inline]
    pub fn into_residual(mut self) -> RenderResidual {
        let res = RenderResidual {
            mask: self.mask,
            next_gc: self.next_gc,
            brushes: Some(self.brushes.take().expect("NPP")),
            width: self.width,
            height: self.height,
        };
        mem::forget(self);
        res
    }

    /// Collect garbage - determine which brushes we probably aren't using, and delete them.
    #[inline]
    fn garbage_collection(&mut self) -> TinyVec<[PixmapPicture; 3]> {
        // todo
        TinyVec::new()
    }
}

impl<'dpy, Dpy: DisplayLike> RenderBreadxSurface<'dpy, Dpy>
where
    Dpy::Connection: Connection,
{
    /// Create a new RenderBreadxSurface from residiual leftover.
    #[inline]
    pub fn from_residual<Target: Into<Drawable>>(
        display: &'dpy mut RenderDisplay<Dpy>,
        picture: Picture,
        parent: Target,
        width: u16,
        height: u16,
        mut residual: RenderResidual,
    ) -> crate::Result<Self> {
        let old_checked = display.checked();
        display.set_checked(true);
        Ok(Self {
            width,
            height,
            display,
            old_checked,
            parent: parent.into(),
            target: picture,
            mask: residual.mask,
            next_gc: residual.next_gc,
            brushes: residual.brushes.take(),
            dropper: Dropper::sync_dropper,
        })
    }

    #[inline]
    pub fn new<Target: Into<Drawable>>(
        display: &'dpy mut RenderDisplay<Dpy>,
        picture: Picture,
        parent: Target,
        width: u16,
        height: u16,
    ) -> crate::Result<Self> {
        // open up the mask picture
        let format = display
            .find_standard_format(StandardFormat::A8)
            .expect("A8 format not available");
        let mask_pixmap = display
            .display_mut()
            .create_pixmap(parent, width, height, 8)?;
        let mask = PixmapPicture {
            pixmap: mask_pixmap,
            picture: display.create_picture(mask_pixmap, format, Default::default()),
        };
    }
}

impl<'dpy, Dpy: DisplayLike> RenderBreadxSurface<'dpy, Dpy> where
    Dpy::Connection: AsyncConnection + Send
{
}

struct Dropper<'dpy, Dpy>(&'dpy Dpy);
