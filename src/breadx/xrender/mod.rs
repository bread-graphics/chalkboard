// MIT/Apache2 License

use crate::{
    fill::FillRule,
    gradient::Gradient,
    surface::{Surface, SurfaceFeatures},
    util::DebugContainer,
    Color,
};
use breadx::{
    auto::{
        render::{
            Color as XrColor, Fixed, Linefix, PictOp, Pictformat, Picture, Pointfix, Repeat,
            Trapezoid, Triangle,
        },
        xproto::{Rectangle as XRectangle, Window},
    },
    display::{prelude::*, Display, DisplayBase},
    render::{
        double_to_fixed, fixed_to_double, tesselate_shape, PictureParameters, RenderDisplay,
        StandardFormat,
    },
    Drawable, Pixmap,
};
use lyon_geom::{Angle, LineSegment, Point, Rect, Size};
use lyon_path::{Path, PathBuffer, PathBufferSlice, PathEvent, PathSlice};
use lyon_tessellation::{
    BuffersBuilder, FillTessellator, FillVertex, FillVertexConstructor, StrokeOption,
    StrokeTessellator, StrokeVertex, StrokeVertexConstructor, VertexBuffers, LineCap, LineJoin,
};
use std::{
    array::IntoIter as ArrayIter,
    cmp,
    collections::hash_map::{Entry, HashMap},
    iter,
    mem::{self, ManuallyDrop},
};
use tinyvec::TinyVec;

#[cfg(feature = "async")]
use breadx::display::AsyncDisplay;
#[cfg(feature = "async")]
use futures_lite::future;

mod brushes;
use brushes::Brushes;

const FEATURES: SurfaceFeatures = SurfaceFeatures {
    gradients: true,
    floats: true,
};
const XCLR_TRANS: XrColor = XrColor {
    red: 0,
    green: 0,
    blue: 0,
    alpha: 0,
};
const XCLR_BLACK: XrColor = XrColor {
    red: 0,
    green: 0,
    blue: 0,
    alpha: 0xFFFF,
};
const XCLR_WHITE: XrColor = XrColor {
    red: 0xFFFF,
    green: 0xFFFF,
    blue: 0xFFFF,
    alpha: 0xFFFF,
};

/// XRender-based BreadX surface. This is preferred over the XProto fallback, but it is generally preferred to
/// use GL rendering on systems that support it.
#[derive(Debug)]
pub struct RenderBreadxSurface<'dpy, Dpy: ?Sized> {
    // display
    display: &'dpy mut RenderDisplay<Dpy>,
    old_checked: bool,

    // drawable to create pixmaps on
    parent: Window,

    // the picture we draw on top of
    target: Picture,

    // width, height and depth of the drawable
    width: u16,
    height: u16,
    depth: u8,

    // cached format for a8 images
    a8_format: Pictformat,
    // cached format for the window
    window_format: Pictformat,

    // we draw shapes onto this picture to use as a mask
    mask: PixmapPicture,

    // a 1x1 image containing solid black, used for drawing shapes on the mask
    solid: PixmapPicture,

    // brushes associated with fill rules
    brushes: Option<Brushes>,

    // stroke color and fill rule
    stroke_color: XrColor,
    fill: FillRule,
    line_width: i32,

    tesselation: Option<Tesselation>,

    // emergency drop mechanism, if free() isnt called
    dropper: DebugContainer<fn(&mut RenderBreadxSurface<'dpy, Dpy>)>,
}

/// Tesselation helper struct.
struct Tesselation {
    // vertex buffers for path tesselation
    buffers: VertexBuffers<Pointfix, usize>,

    // cached tesselators
    fill_tesselator: FillTessellator,
    stroke_tesselator: StrokeTessellator,
}

struct PointfixCvt;

impl FillVertexConstructor<Pointfix> for PointfixCvt {
    fn new_vertex(&mut self, vert: FillVertex<'_>) -> Pointfix {
        let p = vert.position();
        Pointfix {
            x: double_to_fixed(p.x.into()),
            y: double_to_fixed(p.y.into()),
        }
    }
}

impl StrokeVertexConstructor<Pointfix> for PointfixCvt {
    fn new_vertex(&mut self, vert: StrokeVertex<'_>) -> Pointfix {
        let p = vert.position();
        Pointfix {
            x: double_to_fixed(p.x.into()),
            y: double_to_fixed(p.y.into()),
        }
    }
}

impl<'dpy, Dpy: ?Sized> Drop for RenderBreadxSurface<'dpy, Dpy> {
    #[inline]
    fn drop(&mut self) {
        log::warn!("It is preferred to call free() or free_async() rather than dropping RenderBreadxSurface");
        (self.dropper)(self)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub(crate) enum FillRuleKey {
    Color(XrColor),
    LinearGradient(Gradient<'static>, Angle, i32, i32),
    RadialGradient(Gradient<'static>, i32, i32),
    ConicalGradient(Gradient<'static>, i32, i32),
}

/// Residual from the RenderBreadxSurface, used to save space.
#[derive(Debug)]
pub struct RenderResidual {
    mask: PixmapPicture,
    solid: PixmapPicture,
    brushes: Option<Brushes>,
    a8_format: Pictformat,
    window_format: Pictformat,
    width: u16,
    height: u16,
    depth: u8,
    tesselation: Option<Tesselation>,
}

impl RenderResidual {
    #[inline]
    pub fn free<Dpy: Display + ?Sized>(mut self, display: &mut Dpy) -> crate::Result {
        self.mask.free(display)?;
        self.solid.free(display)?;
        self.brushes.take().unwrap().free(display)?;
        let _ = self.tesselation.take();
        mem::forget(self);
        Ok(())
    }

    #[cfg(feature = "async")]
    #[inline]
    pub async fn free_async<Dpy: AsyncDisplay + ?Sized>(
        mut self,
        display: &mut Dpy,
    ) -> crate::Result {
        self.mask.free_async(display).await?;
        self.solid.free_async(display).await?;
        self.brushes.take().unwrap().free_async(display).await?;
        let _ = self.tesselation.take();
        mem::forget(self);
        Ok(())
    }
}

impl Drop for RenderResidual {
    #[inline]
    fn drop(&mut self) {
        log::error!("Dropping RenderResidual without calling free() leaks memory!");
    }
}

/// Combines a pixmap and a picture in one.
#[derive(Debug, Copy, Clone, Default)]
pub(crate) struct PixmapPicture {
    pixmap: Pixmap,
    picture: Picture,
}

impl PixmapPicture {
    #[inline]
    fn free<Dpy: Display + ?Sized>(self, display: &mut Dpy) -> crate::Result {
        self.picture.free(display)?;
        self.pixmap.free(display)?;
        Ok(())
    }

    #[cfg(feature = "async")]
    #[inline]
    async fn free_async<Dpy: AsyncDisplay + ?Sized>(self, display: &mut Dpy) -> crate::Result {
        self.picture.free_async(display).await?;
        self.pixmap.free_async(display).await?;
        Ok(())
    }

    #[inline]
    fn new<Dpy: Display + ?Sized>(
        display: &mut RenderDisplay<Dpy>,
        width: u16,
        height: u16,
        color: XrColor,
        parent: Drawable,
        repeat: bool,
        format: StandardFormat,
        depth: u8,
    ) -> crate::Result<PixmapPicture> {
        let pixmap = display
            .inner_mut()
            .create_pixmap(parent, width, height, depth.into())?;
        let format = display
            .find_standard_format(format)
            .expect("Format not available");
        let params = if repeat {
            PictureParameters {
                repeat: Some(Repeat::Normal),
                ..Default::default()
            }
        } else {
            Default::default()
        };
        let pp = PixmapPicture {
            pixmap,
            picture: display.create_picture(pixmap, format, params)?,
        };

        log::debug!("Filling rectangles for pixmap picture: {:?}", pp.picture);
        pp.picture.fill_rectangles(
            display.inner_mut(),
            PictOp::Over,
            color,
            [XRectangle {
                x: 0,
                y: 0,
                width,
                height,
            }]
            .as_ref(),
        )?;

        Ok(pp)
    }

    #[cfg(feature = "async")]
    #[inline]
    async fn new_async<Dpy: AsyncDisplay + ?Sized>(
        display: &mut RenderDisplay<Dpy>,
        width: u16,
        height: u16,
        color: XrColor,
        parent: Drawable,
        repeat: bool,
        format: StandardFormat,
        depth: u8,
    ) -> crate::Result<PixmapPicture> {
        let pixmap = display
            .inner_mut()
            .create_pixmap_async(parent, width, height, dpeth.into())
            .await?;
        let format = display
            .find_standard_format(format)
            .expect("Format not available");
        let pp = PixmapPicture {
            pixmap,
            picture: display
                .create_picture_async(pixmap, format, Default::default())
                .await?,
        };

        pp.picture
            .fill_rectangles_async(
                display.inner_mut(),
                PictOp::Over,
                color,
                &[XRectangle {
                    x: 0,
                    y: 0,
                    width,
                    height,
                }],
            )
            .await?;

        Ok(pp)
    }

    #[inline]
    fn new_a8<Dpy: Display + ?Sized>(
        display: &mut RenderDisplay<Dpy>,
        width: u16,
        height: u16,
        color: XrColor,
        parent: Drawable,
        repeat: bool,
    ) -> crate::Result<PixmapPicture> {
        Self::new(
            display,
            width,
            height,
            color,
            parent,
            repeat,
            StandardFormat::A8,
            8,
        )
    }

    #[cfg(feature = "async")]
    #[inline]
    async fn new_a8_async<Dpy: AsyncDisplay + ?Sized>(
        display: &mut RenderDisplay<Dpy>,
        width: u16,
        height: u16,
        color: XrColor,
        parent: Drawable,
        repeat: bool,
    ) -> crate::Result<PixmapPicture> {
        Self::new_async(
            display,
            width,
            height,
            color,
            parent,
            repeat,
            StandardFormat::A8,
            8,
        )
        .await
    }

    #[inline]
    fn new_argb32<Dpy: Display + ?Sized>(
        display: &mut RenderDisplay<Dpy>,
        width: u16,
        height: u16,
        color: XrColor,
        parent: Drawable,
        repeat: bool,
    ) -> crate::Result<PixmapPicture> {
        Self::new(
            display,
            width,
            height,
            color,
            parent,
            repeat,
            StandardFormat::Argb32,
            32,
        )
    }

    #[cfg(feature = "async")]
    #[inline]
    async fn new_argb32_async<Dpy: AsyncDisplay + ?Sized>(
        display: &mut RenderDisplay<Dpy>,
        width: u16,
        height: u16,
        color: XrColor,
        parent: Drawable,
        repeat: bool,
    ) -> crate::Result<PixmapPicture> {
        Self::new_async(
            display,
            width,
            height,
            color,
            parent,
            repeat,
            StandardFormat::Argb32,
            32,
        )
        .await
    }
}

/// A picture that may be associated with a pixmap.
#[derive(Debug, Copy, Clone)]
pub(crate) struct MaybePixmapPicture {
    pub(crate) picture: Picture,
    pub(crate) pixmap: Option<Pixmap>,
}

impl MaybePixmapPicture {
    #[inline]
    fn free<Dpy: Display + ?Sized>(self, display: &mut Dpy) -> crate::Result {
        self.picture.free(display)?;
        if let Some(pixmap) = self.pixmap {
            pixmap.free(display)?;
        }

        Ok(())
    }

    #[cfg(feature = "async")]
    #[inline]
    async fn free_async<Dpy: AsyncDisplay + ?Sized>(self, display: &mut Dpy) -> crate::Result {
        self.picture.free_async(display).await?;
        if let Some(pixmap) = self.pixmap {
            pixmap.free_async(display).await?;
        }

        Ok(())
    }

    #[inline]
    fn picture(self) -> Picture {
        self.picture
    }
}

impl From<PixmapPicture> for MaybePixmapPicture {
    #[inline]
    fn from(pp: PixmapPicture) -> Self {
        Self {
            picture: pp.picture,
            pixmap: Some(pp.pixmap),
        }
    }
}

impl From<Picture> for MaybePixmapPicture {
    #[inline]
    fn from(p: Picture) -> Self {
        Self {
            picture: p,
            pixmap: None,
        }
    }
}

impl<'dpy, Dpy: ?Sized> RenderBreadxSurface<'dpy, Dpy> {
    /// Convert this RenderBreadxSurface into the residual.
    #[inline]
    pub fn into_residual(mut self) -> RenderResidual {
        let res = RenderResidual {
            mask: self.mask,
            solid: self.solid,
            brushes: Some(self.brushes.take().expect("NPP")),
            width: self.width,
            height: self.height,
            depth: self.depth,
            window_format: self.window_format,
            a8_format: self.a8_format,
            tesselation: Some(self.tesselation.take().expect("NPP")),
        };
        mem::forget(self);
        res
    }
}

impl<'dpy, Dpy: Display + ?Sized> RenderBreadxSurface<'dpy, Dpy> {
    /// Create a new RenderBreadxSurface from residiual leftover.
    #[inline]
    pub fn from_residual(
        display: &'dpy mut RenderDisplay<Dpy>,
        picture: Picture,
        parent: Window,
        width: u16,
        height: u16,
        mut residual: RenderResidual,
    ) -> crate::Result<Self> {
        let old_checked = display.inner_mut().checked();
        display.inner_mut().set_checked(false);

        // if the width and height doesn't match up, create a new mask
        if width != residual.width || height != residual.height {
            residual.mask.free(display.inner_mut())?;
            residual.mask =
                PixmapPicture::new_a8(display, width, height, XCLR_TRANS, parent.into(), false)?;
        }

        let this = Self {
            width,
            height,
            depth: residual.depth,
            display,
            old_checked,
            parent: parent,
            a8_format: residual.a8_format,
            window_format: residual.window_format,
            target: picture,
            mask: residual.mask,
            solid: residual.solid,
            stroke_color: XCLR_BLACK,
            fill: FillRule::SolidColor(Color::BLACK),
            line_width: 1,
            brushes: residual.brushes.take(),
            tesselation: residual.tesselation.take(),
            dropper: DebugContainer::new(Dropper::<'dpy, Dpy>::sync_dropper),
        };

        mem::forget(residual);

        Ok(this)
    }

    #[inline]
    pub fn new(
        display: &'dpy mut RenderDisplay<Dpy>,
        picture: Picture,
        parent: Window,
        width: u16,
        height: u16,
        depth: u8,
    ) -> crate::Result<Self> {
        let mask = PixmapPicture::new_a8(display, width, height, XCLR_TRANS, parent.into(), false)?;
        let solid = PixmapPicture::new_a8(display, 1, 1, XCLR_WHITE, parent.into(), true)?;

        let window_attrs = parent.window_attributes_immediate(display)?;
        let window_visual = window_attrs.visual;
        let window_visual = display
            .visual_id_to_visual(window_visual)
            .expect("Window visual does not exist");
        let window_format = display
            .find_visual_format(window_visual)
            .expect("Window format does not exist");

        let a8_format = display
            .find_standard_format(StandardFormat::A8)
            .expect("No A8 format");

        Self::from_residual(
            display,
            picture,
            parent,
            width,
            height,
            RenderResidual {
                mask,
                solid,
                brushes: Some(Brushes::new()),
                width,
                height,
                depth,
                window_format,
                a8_format,
                tesselation: Some(Tesselation {
                    vertex_buffers: VertexBuffers::new(),
                    fill: FillTessellator::new(),
                    stroke: StrokeTessellator::new(),
                }),
            },
        )
    }

    #[inline]
    fn free_internal(&mut self) -> crate::Result {
        self.mask.free(self.display.inner_mut())?;
        self.solid.free(self.display.inner_mut())?;
        self.brushes
            .take()
            .unwrap()
            .free(self.display.inner_mut())?;
        self.display.inner_mut().set_checked(self.old_checked);
        Ok(())
    }

    /// Deallocate this renderer's resources.
    #[inline]
    pub fn free(mut self) -> crate::Result {
        let res = self.free_internal();
        mem::forget(self);
        res
    }

    /// Get the picture necessary to act as a source for a stroke operation.
    #[inline]
    fn stroke_picture(&mut self) -> crate::Result<Picture> {
        // lines are just special fill polygons, so we take the fill picture
        self.brushes.as_mut().unwrap().fill(
            &mut self.display,
            self.parent.into(),
            self.depth,
            self.window_format,
            FillRuleKey::Color(self.stroke_color),
        )
    }

    /// Get the picture necessary to act as a source for a fill operation.
    #[inline]
    fn fill_picture(&mut self, width: i32, height: i32) -> crate::Result<Picture> {
        let key = match &self.fill {
            FillRule::SolidColor(clr) => FillRuleKey::Color(cvt_color(*clr)),
            FillRule::LinearGradient(grad, angle) => {
                FillRuleKey::LinearGradient(grad.to_owned(), *angle, width, height)
            }
            FillRule::RadialGradient(grad) => {
                FillRuleKey::RadialGradient(grad.to_owned(), width, height)
            }
            FillRule::ConicalGradient(grad) => {
                FillRuleKey::ConicalGradient(grad.to_owned(), width, height)
            }
        };

        self.brushes.as_mut().unwrap().fill(
            &mut self.display,
            self.parent.into(),
            self.depth,
            self.window_format,
            key,
        )
    }

    #[inline]
    fn fill_triangles(
        &mut self,
        triangles: Vec<Triangle>,
        source: Picture,
        source_x: i16,
        source_y: i16,
    ) -> crate::Result {
        if triangles.is_empty() {
            return Ok(());
        }

        // clear the mask
        self.mask.picture.fill_rectangles(
            self.display.inner_mut(),
            PictOp::Over,
            XCLR_TRANS,
            [XRectangle {
                x: 0,
                y: 0,
                width: self.width,
                height: self.height,
            }]
            .as_ref(),
        )?;

        // draw trapezoids onto the mask
        self.mask.picture.triangles(
            self.display.inner_mut(),
            PictOp::Over,
            self.solid.picture,
            self.a8_format,
            0,
            0,
            triangles,
        )?;

        // use the mask to copy the trapezoids and the desired color onto the destination picture
        source.composite(
            self.display.inner_mut(),
            PictOp::Over,
            self.mask.picture,
            self.target,
            source_x,
            source_y,
            0,
            0,
            0,
            0,
            self.width,
            self.height,
        )?;

        // should be done now
        Ok(())
    }

    #[inline]
    fn draw_lines_internal<I: IntoIterator<Item = LineSegment<f32>>>(
        &mut self,
        lines: I,
    ) -> crate::Result {
        let src = self.stroke_picture()?;
        let line_width = self.line_width;
        let triangles: Vec<Triangle> = lines
            .into_iter()
            .flat_map(|l| line_to_triangles(l, line_width as _))
            .collect();
        self.fill_triangles(triangles, src, 0, 0)
    }

    #[inline]
    fn fill_rectangles_internal<I: IntoIterator<Item = Rect<f32>>>(
        &mut self,
        rects: I,
    ) -> crate::Result {
        // fast path: if all we have are solid colors, just use fill_rectangles()
        if let FillRule::SolidColor(clr) = self.fill {
            let clr = cvt_color(clr);
            let rects: Vec<XRectangle> = rects
                .into_iter()
                .map(
                    |Rect {
                         origin: Point { x, y },
                         size: Size { width, height },
                     }| XRectangle {
                        x: x as _,
                        y: y as _,
                        width: width as _,
                        height: height as _,
                    },
                )
                .collect();
            self.target
                .fill_rectangles(self.display.inner_mut(), PictOp::Src, clr, rects)?;
            return Ok(());
        }

        // slow path: convert every rectangle to two triangles and then composite it
        let rects: Vec<Rect<f32>> = rects
            .into_iter()
            .filter(
                |Rect {
                     size: Size { width, height },
                 }| width != 0 && height != 0,
            )
            .collect();
        if rects.is_empty() {
            return Ok(());
        }

        // get the min/max x and min/max y
        let min_x = *rects
            .iter()
            .map(
                |Rect {
                     origin: Point { x, .. },
                     ..
                 }| x,
            )
            .min()
            .unwrap();
        let max_x = *rects
            .iter()
            .map(
                |Rect {
                     origin: Point { x, .. },
                     size: Size { width, .. },
                     ..
                 }| x + width,
            )
            .max()
            .unwrap();
        let min_y = *rects
            .iter()
            .map(
                |Rect {
                     origin: Point { y, .. },
                     ..
                 }| y,
            )
            .min()
            .unwrap();
        let max_y = *rects
            .iter()
            .map(
                |Rect {
                     origin: Point { y, .. },
                     size: Size { height, .. },
                     ..
                 }| y + height,
            )
            .max()
            .unwrap();

        let triangles: Vec<Triangle> = rects
            .into_iter()
            .flat_map(
                |Rect {
                     origin: Point { x, y },
                     size: Size { width, height },
                     ..
                 }| {
                    let x1 = double_to_fixed(x - min_x);
                    let y1 = double_to_fixed(y - min_y);
                    let x2 = double_to_fixed(x + width - min_x);
                    let y2 = double_to_fixed(y + height - min_y);

                    ArrayIter::new([
                        Triangle {
                            p1: Pointfix { x: x1, y: y1 },
                            p2: Pointfix { x: x2, y: y1 },
                            p3: Pointfix { x: x1, y: y2 },
                        },
                        Triangle {
                            p1: Pointfix { x: x2, y: y1 },
                            p2: Pointfix { x: x2, y: y2 },
                            p3: Pointfix { x: x1, y: y2 },
                        },
                    ])
                },
            )
            .collect();

        let fill = self.fill_picture(max_x - min_x, max_y - min_y)?;
        self.fill_triangles(triangles, fill, -min_x as i16, -min_y as i16)
    }

    #[inline]
    fn tesselate_stroke_path(&mut self, path: impl Iterator<Item = PathEvent>) -> Vec<Triangle> {
        // use lyon_tesselate to tesselate the stroke
        let stroke_options = StrokeOption {
            start_cap: LineCap::Butt,
            end_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            line_width: self.line_width as f32,
            ..Default::default()
        };

        let mut tesselate = self.tesselate.as_mut().expect("NPP");

        tesselate.buffers.vertices.clear();
        tesselate.buffers.indices.clear();

        let mut buffer = BuffersBuilder::new(&mut tesselate.buffers, PointfixCvt);
        tesselate
            .stroke
            .tesselate(path, &stroke_options, &mut buffer);

        let vertices = mem::take(&mut tesselate.buffers.vertices);
        let triangles: Vec<Triangle> = tesselate
            .buffers
            .indices
            .chunks_exact(3)
            .map(move |chunk| Triangle {
                p1: vertices[chunk[0]],
                p2: vertices[chunk[1]],
                p3: vertices[chunk[2]],
            })
            .collect();

        triangles
    }
    
    #[inline]
    fn stroke_path(&mut self, path: impl Iterator<Item = PathEvent>) -> crate::Result {
        let triangles = self.tessleate_stroke_path(path);
        let fill = self.stroke_picture()?;
        self.fill_triangles(triangles, fill, 0, 0)
    }

    #[inline]
    fn tesselate_fill_path(&mut self, path: impl Iterator<Item = PathEvent>) -> FillPathInfo {
        let mut tesselate = self.tesselate.as_mut().expect("NPP");
        tesselate.buffers.vertices.clear();
        tesselate.buffers.indices.clear();

        let mut buffer = BuffersBuilder::new(&mut tesselate.buffers, PointfixCvt);
        tesselate
            .fill
            .tesselate(path, &Default::default(), &mut buffer);

        let vertices = mem::take(&mut tesselate.buffers.vertices);

        if vertices.len() < 3 || tesselate.buffer.indices.len() < 3 {
            return FillPathInfo::default();
        }

        let x_iter = vertices.iter().map(|Pointfix { x, .. }| x);
        let min_x = x_iter.clone().min().unwrap();
        let max_x = x_iter.max().unwrap();

        let y_iter = vertices.iter().map(|Pointfix { y, .. }| y);
        let min_y = y_iter.clone().min().unwrap();
        let max_y = y_iter.max().unwrap();

        let triangles: Vec<Triangle> = tesselate
            .buffers()
            .indices
            .chunks_exact(3)
            .map(move |chunk| Triangle {
                p1: vertices[chunk[0]],
                p2: vertices[chunk[1]],
                p3: vertices[chunk[2]],
            })
            .collect();

        FillPathInfo { triangles, min_x, max_x, min_y, max_y }
    }

    #[inline]
    fn fill_path(&mut self, path: impl Iterator<Item = PathEvent>) -> crate::Result {
        let FillPathInfo { triangles, min_x, min_y, max_x, max_y } = self.tesselate_fill_path(path);
        if triangles.is_empty() { return Ok(()); }

        let width = fixed_to_double(max_x - min_x);
        let height = fixed_to_double(max_y - min_y);

        let fill = self.fill_picture(width as i32, height as i32)?;
        self.fill_triangles(
            triangles,
            fill,
            -fixed_to_double(min_x) as i16,
            -fixed_to_double(min_y) as i16,
        )
    }
}

#[derive(Default)]
struct FillPathInfo {
    triangles: Vec<Triangle>,
    min_x: f32, max_x: f32, min_y: f32, max_y: f32,
}

#[cfg(feature = "async")]
impl<'dpy, Dpy: AsyncDisplay + ?Sized> RenderBreadxSurface<'dpy, Dpy> {
    /// Create a new RenderBreadxSurface from residiual leftover, async redox.
    #[inline]
    pub async fn from_residual_async<Target: Into<Drawable>>(
        display: &'dpy mut RenderDisplay<Dpy>,
        picture: Picture,
        parent: Target,
        width: u16,
        height: u16,
        mut residual: RenderResidual,
    ) -> crate::Result<RenderBreadxSurface<'dpy, Dpy>> {
        let old_checked = display.inner_mut().checked();
        display.inner_mut().set_checked(true);
        let parent: Drawable = parent.into();

        // if the width and height doesn't match up, create a new mask
        if width != residual.width || height != residual.height {
            residual.mask.free_async(display.inner_mut()).await?;
            residual.mask =
                PixmapPicture::new_a8_async(display, width, height, XCLR_TRANS, parent, false)
                    .await?;
        }

        Ok(Self {
            width,
            height,
            display,
            old_checked,
            parent,
            target: picture,
            mask: residual.mask,
            solid: residual.solid,
            stroke: Color::BLACK,
            stroke_applied: true,
            fill: FillRule::SolidColor(Color::BLACK),
            next_gc: residual.next_gc,
            brushes: residual.brushes.take(),
            dropper: DebugContainer::new(Dropper::<'dpy, Dpy>::async_dropper),
        })
    }

    #[inline]
    pub async fn new_async<Target: Into<Drawable>>(
        display: &'dpy mut RenderDisplay<Dpy>,
        picture: Picture,
        parent: Target,
        width: u16,
        height: u16,
    ) -> crate::Result<RenderBreadxSurface<'dpy, Dpy>> {
        let parent: Drawable = parent.into();
        let mask =
            PixmapPicture::new_a8_async(display, width, height, XCLR_TRANS, parent, false).await?;
        let solid =
            PixmapPicture::new_argb32_async(display, width, height, XCLR_WHITE, parent, true)
                .await?;
        Self::from_residual_async(
            display,
            picture,
            parent,
            width,
            height,
            RenderResidual {
                mask,
                solid,
                next_gc: 32,
                brushes: Some(HashMap::new()),
                width,
                height,
            },
        )
        .await
    }

    #[inline]
    async fn free_internal_async(&mut self) -> crate::Result {
        self.mask.free_async(self.display.inner_mut()).await?;
        for v in self.brushes.take().unwrap().values() {
            v.free_async(self.display.inner_mut()).await?;
        }
        self.display.inner_mut().set_checked(self.old_checked);
        Ok(())
    }

    /// Deallocate this renderer's resources.
    #[inline]
    pub async fn free_async(mut self) -> crate::Result {
        let res = self.free_internal_async().await;
        mem::forget(self);
        res
    }
}

impl<'dpy, Dpy: Display + ?Sized> Surface for RenderBreadxSurface<'dpy, Dpy> {
    #[inline]
    fn features(&self) -> SurfaceFeatures {
        FEATURES
    }

    #[inline]
    fn set_stroke(&mut self, color: Color) -> crate::Result {
        self.stroke_color = cvt_color(color);
        Ok(())
    }

    #[inline]
    fn set_fill(&mut self, rule: FillRule) -> crate::Result {
        self.fill = rule;
        Ok(())
    }

    #[inline]
    fn set_line_width(&mut self, width: usize) -> crate::Result {
        self.line_width = width as _;
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> crate::Result {
        self.display.inner_mut().synchronize()?;
        Ok(())
    }

    #[inline]
    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> crate::Result {
        self.draw_lines_internal(iter::once(LineSegment {
            to: Point { x: x1, y: y1 },
            from: Point { x: x2, y: y2 },
        }))
    }

    #[inline]
    fn draw_lines(&mut self, lines: &[LineSegment<f32>]) -> crate::Result {
        self.draw_lines_internal(lines.iter().copied())
    }

    #[inline]
    fn draw_path(&mut self, path: PathSlice<'_>) -> crate::Result {
        self.stroke_path(path.iter())
    }

    #[inline]
    fn draw_path_owned(&mut self, path: Path) -> crate::Result {
        self.stroke_path(path.iter())
    }

    #[inline]
    fn draw_paths(&mut self, paths: PathBufferSlice<'_>) -> crate::Result {
        paths
            .indices()
            .try_for_each(|path| self.stroke_path(paths.get(path).iter()))
    }

    #[inline]
    fn draw_paths_owned(&mut self, paths: PathBuffer) -> crate::Result {
        paths
            .indices()
            .try_for_each(|path| self.stroke_path(paths.get(path).iter()))
    }

    #[inline]
    fn fill_polygon(&mut self, points: &[Point]) -> crate::Result {
        if points.len() < 3 {
            return Ok(());
        }

        let mut builder = Path::builder();
        builder.begin(points[0]);
        let mut builder = points
            .iter()
            .skip(1)
            .copied()
            .fold(builder, |mut builder, point| {
                builder.line_to(point);
                builder
            });
        builder.close();
        let path = builder.build();

        self.fill_path(path.iter())
    }

    #[inline]
    fn fill_path(&mut self, path: PathSlice<'_>) -> crate::Result {
        self.fill_path(path.iter())
    }

    #[inline]
    fn fill_path_owned(&mut self, path: Path) -> crate::Result {
        self.fill_path(path.iter())
    }

    #[inline]
    fn fill_paths_owned(&mut self, paths: PathBuffer) -> crate::Result {
        paths
            .indices()
            .try_for_each(|index| self.fill_path(paths.get(index).iter()))
    }

    #[inline]
    fn fill_paths(&mut self, paths: PathBufferSlice<'_>) -> crate::Result {
        paths
            .indices()
            .try_for_each(|index| self.fill_path(paths.get(index).iter()))
    }

    #[inline]
    fn fill_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32) -> crate::Result {
        self.fill_rectangles_internal(iter::once(Rect {
            origin: Point { x, y },
            size: Size { width, height },
        }))
    }

    #[inline]
    fn fill_rectangles(&mut self, rects: &[Rect<f32>]) -> crate::Result {
        self.fill_rectangles_internal(rects.iter().copied())
    }
}

#[inline]
fn line_to_triangles(line: LineSegment<f32>, width: usize) -> impl Iterator<Item = Triangle> {
    let width = width as f64;
    // figure out at which angle the line segment is at
    let angle = ((line.y2 - line.y1) as f64).atan2((line.x2 - line.x1) as f64);
    let dx = angle.cos() * (width / 2.0);
    let dy = angle.sin() * (width / 2.0);
    let x1 = line.x1 as f64;
    let x2 = line.x2 as f64;
    let y1 = line.y1 as f64;
    let y2 = line.y2 as f64;

    let t1 = double_to_fixed(x1 + dx);
    let l1 = double_to_fixed(y1 + dy);
    let b1 = double_to_fixed(x1 - dx);
    let r1 = double_to_fixed(y1 - dy);

    let t2 = double_to_fixed(x2 + dx);
    let l2 = double_to_fixed(y2 + dy);
    let b2 = double_to_fixed(x2 - dx);
    let r2 = double_to_fixed(y2 - dy);

    ArrayIter::new([
        Triangle {
            p1: Pointfix { x: t1, y: l1 },
            p2: Pointfix { x: t2, y: l2 },
            p3: Pointfix { x: b1, y: r1 },
        },
        Triangle {
            p1: Pointfix { x: t2, y: l2 },
            p2: Pointfix { x: b2, y: r2 },
            p3: Pointfix { x: b1, y: r1 },
        },
    ])
}

struct Dropper<'dpy, Dpy: ?Sized>(&'dpy Dpy);

impl<'dpy, Dpy: Display + ?Sized> Dropper<'dpy, Dpy> {
    fn sync_dropper(this: &mut RenderBreadxSurface<'dpy, Dpy>) {
        if let Err(e) = this.free_internal() {
            log::error!("Failed to free RenderBreadxSurface: {:?}", e);
        }
    }
}

#[cfg(feature = "async")]
impl<'dpy, Dpy: AsyncDisplay + ?Sized> Dropper<'dpy, Dpy> {
    fn async_dropper(this: &mut RenderBreadxSurface<'dpy, Dpy>) {
        future::block_on(async {
            if let Err(e) = this.free_internal_async().await {
                log::error!("Failed to free RenderBreadxSurface: {:?}", e);
            }
        });
    }
}

#[inline]
pub(crate) fn cvt_color(color: Color) -> XrColor {
    let (red, green, blue, alpha) = color.clamp_u16();
    XrColor {
        red,
        green,
        blue,
        alpha,
    }
}

#[inline]
fn cvt_rect(rect: Rect<f32>) -> XRectangle {
    XRectangle {
        x: rect.x as _,
        y: rect.y as _,
        width: rect.width as _,
        height: rect.height as _,
    }
}
