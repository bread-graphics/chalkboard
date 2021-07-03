// MIT/Apache2 License

use crate::{
    color::Color,
    fill::FillRule,
    geometry::{Angle, Line, Point, Rectangle},
    gradient::Gradient,
    surface::{Surface, SurfaceFeatures},
    util::DebugContainer,
};
use breadx::{
    auto::{
        render::{
            Color as XrColor, Fixed, Linefix, PictOp, Pictformat, Picture, Pointfix, Trapezoid,
        },
        xproto::Rectangle as XRectangle,
    },
    display::{prelude::*, Display, DisplayBase},
    render::{double_to_fixed, tesselate_shape, RenderDisplay, StandardFormat},
    Drawable, Pixmap,
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

const FEATURES: SurfaceFeatures = SurfaceFeatures { gradients: true };
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
    parent: Drawable,

    // target
    target: Picture,
    width: u16,
    height: u16,

    a8_format: Pictformat,

    // we draw shapes onto this picture to use as a mask
    mask: PixmapPicture,
    solid: PixmapPicture,
    stroke: PixmapPicture,

    // brushes associated with fill rules
    brushes: Option<HashMap<FillRuleKey, MaybePixmapPicture>>,

    // stroke color and fill rule
    stroke_color: XrColor,
    stroke_applied: bool,
    fill: FillRule,
    line_width: i32,

    // wait until the next garbage collection
    next_gc: usize,

    // emergency drop mechanism, if free() isnt called
    dropper: DebugContainer<fn(&mut RenderBreadxSurface<'dpy, Dpy>)>,
}

impl<'dpy, Dpy: ?Sized> Drop for RenderBreadxSurface<'dpy, Dpy> {
    #[inline]
    fn drop(&mut self) {
        log::warn!("It is preferred to call free() or free_async() rather than dropping RenderBreadxSurface");
        (self.dropper)(self)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
enum FillRuleKey {
    Color(Color),
    LinearGradient(Gradient, Angle, Rectangle),
    RadialGradient(Gradient, Rectangle),
    ConicalGradient(Gradient, Rectangle),
}

/// Residual from the RenderBreadxSurface, used to save space.
#[derive(Debug)]
pub struct RenderResidual {
    mask: PixmapPicture,
    solid: PixmapPicture,
    stroke: PixmapPicture,
    brushes: Option<HashMap<FillRuleKey, MaybePixmapPicture>>,
    next_gc: usize,
    width: u16,
    height: u16,
}

impl RenderResidual {
    #[inline]
    pub fn free<Dpy: Display + ?Sized>(mut self, display: &mut Dpy) -> crate::Result {
        self.mask.free(display)?;
        self.solid.free(display)?;
        self.stroke.free(display)?;
        self.brushes
            .as_ref()
            .unwrap()
            .values()
            .try_for_each(|val| val.free(display))?;
        self.brushes.take();
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
        self.stroke.free_async(display).await?;
        for val in self.brushes.as_ref().unwrap().values() {
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
        log::error!("Dropping RenderResidual without calling free() leaks memory!");
    }
}

/// Combines a pixmap and a picture in one.
#[derive(Debug, Copy, Clone, Default)]
struct PixmapPicture {
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
        let pp = PixmapPicture {
            pixmap,
            picture: display.create_picture(pixmap, format, Default::default())?,
        };

        log::debug!("Filling rectangles for pixmap picture: {:?}", pp.picture);
        pp.picture.fill_rectangles(
            display.inner_mut(),
            PictOp::Src,
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
                .create_picture_async(parent, format, Default::default())
                .await?,
        };

        pp.picture
            .fill_rectangles_async(
                display.inner_mut(),
                PictOp::Clear,
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
enum MaybePixmapPicture {
    NoPixmap(Picture),
    Pixmap(PixmapPicture),
}

impl MaybePixmapPicture {
    #[inline]
    fn free<Dpy: Display + ?Sized>(self, display: &mut Dpy) -> crate::Result {
        match self {
            Self::NoPixmap(pic) => pic.free(display)?,
            Self::Pixmap(pp) => pp.free(display)?,
        }

        Ok(())
    }

    #[cfg(feature = "async")]
    #[inline]
    async fn free_async<Dpy: AsyncDisplay + ?Sized>(self, display: &mut Dpy) -> crate::Result {
        match self {
            Self::NoPixmap(pic) => pic.free_async(display).await?,
            Self::Pixmap(pp) => pp.free_async(display).await?,
        }

        Ok(())
    }

    #[inline]
    fn picture(self) -> Picture {
        match self {
            Self::NoPixmap(pic) => pic,
            Self::Pixmap(pp) => pp.picture,
        }
    }
}

impl From<PixmapPicture> for MaybePixmapPicture {
    #[inline]
    fn from(pp: PixmapPicture) -> Self {
        Self::Pixmap(pp)
    }
}

impl From<Picture> for MaybePixmapPicture {
    #[inline]
    fn from(p: Picture) -> Self {
        Self::NoPixmap(p)
    }
}

impl<'dpy, Dpy: ?Sized> RenderBreadxSurface<'dpy, Dpy> {
    /// Convert this RenderBreadxSurface into the residual.
    #[inline]
    pub fn into_residual(mut self) -> RenderResidual {
        let res = RenderResidual {
            mask: self.mask,
            solid: self.solid,
            stroke: self.stroke,
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

impl<'dpy, Dpy: Display + ?Sized> RenderBreadxSurface<'dpy, Dpy> {
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
        let old_checked = display.inner_mut().checked();
        display.inner_mut().set_checked(true);
        let parent: Drawable = parent.into();
        let a8_format = display
            .find_standard_format(StandardFormat::A8)
            .expect("No A8 format present");

        // if the width and height doesn't match up, create a new mask
        if width != residual.width || height != residual.height {
            residual.mask.free(display.inner_mut())?;
            residual.mask =
                PixmapPicture::new_a8(display, width, height, XCLR_TRANS, parent, false)?;
        }

        Ok(Self {
            width,
            height,
            display,
            old_checked,
            parent: parent,
            a8_format,
            target: picture,
            mask: residual.mask,
            solid: residual.solid,
            stroke: residual.stroke,
            stroke_color: XCLR_BLACK,
            stroke_applied: true,
            fill: FillRule::SolidColor(Color::BLACK),
            line_width: 1,
            next_gc: residual.next_gc,
            brushes: residual.brushes.take(),
            dropper: DebugContainer::new(Dropper::<'dpy, Dpy>::sync_dropper),
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
        let parent: Drawable = parent.into();
        let mask = PixmapPicture::new_a8(display, width, height, XCLR_TRANS, parent, false)?;
        let solid = PixmapPicture::new_a8(display, width, height, XCLR_BLACK, parent, true)?;
        let stroke = PixmapPicture::new_argb32(display, width, height, XCLR_BLACK, parent, true)?;
        Self::from_residual(
            display,
            picture,
            parent,
            width,
            height,
            RenderResidual {
                mask,
                solid,
                stroke,
                next_gc: 32,
                brushes: Some(HashMap::new()),
                width,
                height,
            },
        )
    }

    #[inline]
    fn free_internal(&mut self) -> crate::Result {
        self.mask.free(self.display.inner_mut())?;
        self.stroke.free(self.display.inner_mut())?;
        self.solid.free(self.display.inner_mut())?;
        self.brushes
            .take()
            .unwrap()
            .values()
            .try_for_each(|v| v.free(self.display.inner_mut()))?;
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
        if !self.stroke_applied {
            let color = self.stroke_color;
            self.stroke.picture.fill_rectangles(
                self.display.inner_mut(),
                PictOp::Src,
                color,
                [XRectangle {
                    x: 0,
                    y: 0,
                    width: self.width as _,
                    height: self.height as _,
                }]
                .as_ref(),
            )?;
            self.stroke_applied = true;
        }
        Ok(self.stroke.picture)
    }

    /// Get the picture necessary to act as a source for a fill operation.
    #[inline]
    fn fill_picture(&mut self, rect: Rectangle) -> crate::Result<Picture> {
        let key = match &self.fill {
            FillRule::SolidColor(clr) => FillRuleKey::Color(*clr),
            FillRule::LinearGradient(grad, angle) => {
                FillRuleKey::LinearGradient(grad.clone(), *angle, rect)
            }
            FillRule::RadialGradient(grad) => FillRuleKey::RadialGradient(grad.clone(), rect),
            FillRule::ConicalGradient(grad) => FillRuleKey::ConicalGradient(grad.clone(), rect),
        };

        match self.brushes.as_mut().unwrap().entry(key) {
            Entry::Occupied(o) => Ok(o.get().picture()),
            Entry::Vacant(v) => match v.key() {
                FillRuleKey::Color(clr) => {
                    let brush = PixmapPicture::new_argb32(
                        self.display,
                        self.width,
                        self.height,
                        cvt_color(*clr),
                        self.parent,
                        true,
                    )?;
                    v.insert(brush.into());
                    Ok(brush.picture)
                }
                FillRuleKey::LinearGradient(grad, angle, rect) => {
                    let Rectangle { x1, y1, x2, y2 } = rect;
                    let width = (x2 - x1).abs();
                    let height = (y2 - y2).abs();
                    let (p1, p2) = rectangle_angle(width as f64, height as f64, *angle);
                    let (stops, color) = gradient_to_stops_and_color(grad);
                    let grad = self.display.create_linear_gradient(
                        p1,
                        p2,
                        stops.as_slice(),
                        color.as_slice(),
                    )?;
                    v.insert(grad.into());
                    Ok(grad)
                }
                FillRuleKey::RadialGradient(grad, rect) => {
                    let Rectangle { x1, y1, x2, y2 } = rect;
                    let width = (x2 - x1).abs();
                    let height = (y2 - y2).abs();
                    let radius = double_to_fixed(width as f64);
                    let scaling = (height as f64) / (width as f64);
                    let c = radius / 2;
                    let cp = Pointfix { x: c, y: c };
                    let (stops, color) = gradient_to_stops_and_color(grad);
                    // create a radial gradient and use transforms to scale it
                    let radial = self.display.create_radial_gradient(
                        cp.clone(),
                        cp,
                        0,
                        radius,
                        stops.as_slice(),
                        color.as_slice(),
                    )?;
                    // TODO: apply transform
                    v.insert(radial.into());
                    Ok(radial)
                }
                FillRuleKey::ConicalGradient(grad, rect) => {
                    let Rectangle { x1, y1, x2, y2 } = rect;
                    let width = (x2 - x1).abs();
                    let height = (y2 - y2).abs();
                    let radius = double_to_fixed(width as f64);
                    let scaling = (height as f64) / (width as f64);
                    let c = radius / 2;
                    let cp = Pointfix { x: c, y: c };
                    let (stops, color) = gradient_to_stops_and_color(grad);
                    // create a radial gradient and use transforms to scale it
                    let conical = self.display.create_conical_gradient(
                        cp,
                        0,
                        stops.as_ref(),
                        color.as_ref(),
                    )?;
                    // TODO: apply transform
                    v.insert(conical.into());
                    Ok(conical)
                }
            },
        }
    }

    #[inline]
    fn fill_trapezoids(&mut self, traps: &[Trapezoid], source: Picture) -> crate::Result {
        // clear the mask
        self.mask.picture.fill_rectangles(
            self.display.inner_mut(),
            PictOp::Src,
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
        self.mask.picture.trapezoids(
            self.display.inner_mut(),
            PictOp::Over,
            self.solid.picture,
            self.a8_format,
            0,
            0,
            traps.as_ref(),
        )?;

        // use the mask to copy the trapezoids and the desired color onto the destination picture
        source.composite(
            self.display.inner_mut(),
            PictOp::Src,
            self.mask.picture,
            self.target,
            0,
            0,
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
    fn draw_lines_internal<I: IntoIterator<Item = Line>>(&mut self, lines: I) -> crate::Result {
        let traps: Vec<Trapezoid> = lines
            .into_iter()
            .flat_map(|l| line_to_trapezoids(l, self.line_width as _))
            .collect();
        let src = self.stroke_picture()?;
        self.fill_trapezoids(&traps, src)
    }

    #[inline]
    fn fill_rectangles_internal<I: IntoIterator<Item = Rectangle>>(
        &mut self,
        rects: I,
    ) -> crate::Result {
        // fast path: if all we have are solid colors, just use fill_rectangles()
        if let FillRule::SolidColor(clr) = self.fill {
            let clr = cvt_color(clr);
            let rects: Vec<XRectangle> = rects
                .into_iter()
                .map(|Rectangle { x1, y1, x2, y2 }| XRectangle {
                    x: x1 as _,
                    y: y1 as _,
                    width: (x2 - x1) as _,
                    height: (y2 - y1) as _,
                })
                .collect();
            self.target
                .fill_rectangles(self.display.inner_mut(), PictOp::Src, clr, &rects)?;
            return Ok(());
        }

        let traps: Vec<Trapezoid> = rects.into_iter().map(|r| rect_to_trapezoid(r)).collect();
        let x1 = traps
            .iter()
            .map(
                |Trapezoid {
                     left:
                         Linefix {
                             p1: Pointfix { x, .. },
                             ..
                         },
                     ..
                 }| *x,
            )
            .min()
            .unwrap();
        let x2 = traps
            .iter()
            .map(
                |Trapezoid {
                     right:
                         Linefix {
                             p1: Pointfix { x, .. },
                             ..
                         },
                     ..
                 }| *x,
            )
            .max()
            .unwrap();
        let width = (x2 - x1) >> 16;

        let y1 = traps.iter().map(|Trapezoid { top, .. }| top).min().unwrap();
        let y2 = traps
            .iter()
            .map(|Trapezoid { bottom, .. }| bottom)
            .max()
            .unwrap();
        let height = (y2 - y1) >> 16;

        let src = self.fill_picture(Rectangle {
            x1: 0,
            y1: 0,
            x2: width as _,
            y2: height as _,
        })?;
        self.fill_trapezoids(&traps, src)
    }
}

// helper function to get stops and color from a gradient
#[inline]
fn gradient_to_stops_and_color(grad: &Gradient) -> (TinyVec<[Fixed; 6]>, TinyVec<[XrColor; 3]>) {
    grad.iter()
        .map(|r| {
            (
                double_to_fixed(r.position.into_inner().into()),
                cvt_color(r.color),
            )
        })
        .unzip()
}

// helper function to get points on the rectangle corresponding to angles
#[inline]
fn rectangle_angle(width: f64, height: f64, angle: Angle) -> (Pointfix, Pointfix) {
    // fast paths
    if angle.approx_eq(Angle::ZERO) || angle.approx_eq(Angle::FULL_CIRCLE) {
        let h2 = double_to_fixed(height / 2.0);
        return (
            Pointfix { x: 0, y: h2 },
            Pointfix {
                x: double_to_fixed(width),
                y: h2,
            },
        );
    } else if angle.approx_eq(Angle::QUARTER_CIRCLE) {
        let w2 = double_to_fixed(width / 2.0);
        return (
            Pointfix { x: w2, y: 0 },
            Pointfix {
                x: w2,
                y: double_to_fixed(height),
            },
        );
    } else if angle.approx_eq(Angle::HALF_CIRCLE) {
        let h2 = double_to_fixed(height / 2.0);
        return (
            Pointfix { x: 0, y: h2 },
            Pointfix {
                x: double_to_fixed(width),
                y: h2,
            },
        );
    } else if angle.approx_eq(Angle::THREE_QUARTERS_CIRCLE) {
        let w2 = double_to_fixed(width / 2.0);
        return (
            Pointfix {
                x: w2,
                y: double_to_fixed(height),
            },
            Pointfix { x: w2, y: 0 },
        );
    }

    // slow path: calculate a point going from the center of the rectangle to the edges. we can do this with
    // the knowledge that the slope, which we can calculate via atan(angle), is y/x. given the center, we can use
    // this to calculate the x at y=height and the y at x=width, and figure out which one fits
    // then do the same at y = 0 and x = 0
    let slope = angle.radians().atan() as f64;
    let xc = width / 2.0;
    let yc = height / 2.0;

    let mut calc_point_at = move |xfix: f64, yfix: f64| -> Pointfix {
        // (y - y1) = m*(x - x1) -> y = y1 + m*(x - x1), where x1 and y1 are the center and x = width or 0
        let y_est = yc + (slope * (xfix - xc));
        // x = x1 + (y - y1)/m
        let x_est = xc + ((yfix - yc) / slope);
        // calculate the results of the estimate
        let y_est_x = xc + ((yfix - y_est) / slope);
        let x_est_y = yc + (slope * (xfix - x_est));

        // one of these estimates will be larger than the rectangle proper, so account for that and choose which
        // one works
        if y_est_x > width {
            Pointfix {
                x: double_to_fixed(x_est),
                y: double_to_fixed(x_est_y),
            }
        } else {
            Pointfix {
                x: double_to_fixed(y_est_x),
                y: double_to_fixed(y_est),
            }
        }
    };

    (calc_point_at(0.0, 0.0), calc_point_at(width, height))
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
        self.stroke_applied = false;
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
    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        self.draw_lines_internal(iter::once(Line { x1, y1, x2, y2 }))
    }

    #[inline]
    fn draw_lines(&mut self, lines: &[Line]) -> crate::Result {
        self.draw_lines_internal(lines.iter().copied())
    }

    // other line-drawing mechanisms should just use draw_lines as a front

    #[inline]
    fn fill_polygon(&mut self, points: &[Point]) -> crate::Result {
        if points.len() < 3 {
            return Ok(());
        }

        let xiter = points.iter().copied().map(|Point { x, .. }| x);
        let x1 = xiter.clone().min().unwrap();
        let x2 = xiter.max().unwrap();

        let yiter = points.iter().copied().map(|Point { y, .. }| y);
        let y1 = yiter.clone().min().unwrap();
        let y2 = yiter.max().unwrap();

        let rect = Rectangle { x1, y1, x2, y2 };

        // translate shapes to polygons
        let points = points.iter().copied().map(|Point { x, y }| Pointfix {
            x: x << 16,
            y: y << 16,
        });
        let traps: Vec<Trapezoid> = tesselate_shape(points);
        let src = self.fill_picture(rect)?;
        self.fill_trapezoids(&traps, src)
    }

    #[inline]
    fn fill_rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        self.fill_rectangles_internal(iter::once(Rectangle { x1, y1, x2, y2 }))
    }

    #[inline]
    fn fill_rectangles(&mut self, rects: &[Rectangle]) -> crate::Result {
        self.fill_rectangles_internal(rects.iter().copied())
    }
}

#[inline]
fn rect_to_trapezoid(rect: Rectangle) -> Trapezoid {
    let x1 = cmp::min(rect.x1, rect.x2);
    let x2 = cmp::max(rect.x1, rect.x2);
    let y1 = cmp::min(rect.y1, rect.y2);
    let y2 = cmp::max(rect.y1, rect.y2);
    let l1 = Linefix {
        p1: Pointfix {
            x: x1 << 16,
            y: y1 << 16,
        },
        p2: Pointfix {
            x: x1 << 16,
            y: y2 << 16,
        },
    };
    let l2 = Linefix {
        p1: Pointfix {
            x: x2 << 16,
            y: y1 << 16,
        },
        p2: Pointfix {
            x: x2 << 16,
            y: y2 << 16,
        },
    };
    Trapezoid {
        left: l1,
        right: l2,
        top: y1 << 16,
        bottom: y2 << 16,
    }
}

#[inline]
fn line_to_trapezoids(line: Line, width: usize) -> Vec<Trapezoid> {
    let width = width as f64;
    // figure out at which angle the line segment is at
    let angle = ((line.y2 - line.y1) as f64).atan2((line.x2 - line.x1) as f64);
    let dx = angle.cos() * (width / 2.0);
    let dy = angle.sin() * (width / 2.0);
    let x1 = line.x1 as f64;
    let x2 = line.x2 as f64;
    let y1 = line.y1 as f64;
    let y2 = line.y2 as f64;

    let rectangle: [Pointfix; 4] = [
        Pointfix {
            x: double_to_fixed(x1 + dx),
            y: double_to_fixed(y1 + dy),
        },
        Pointfix {
            x: double_to_fixed(x2 + dx),
            y: double_to_fixed(y2 + dy),
        },
        Pointfix {
            x: double_to_fixed(x2 - dx),
            y: double_to_fixed(y2 - dy),
        },
        Pointfix {
            x: double_to_fixed(x1 - dx),
            y: double_to_fixed(y1 - dy),
        },
    ];

    tesselate_shape(ArrayIter::new(rectangle))
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
fn cvt_color(color: Color) -> XrColor {
    let (red, green, blue, alpha) = color.clamp_u16();
    XrColor {
        red,
        green,
        blue,
        alpha,
    }
}

#[inline]
fn cvt_rect(rect: Rectangle) -> XRectangle {
    XRectangle {
        x: cmp::min(rect.x1, rect.x2) as _,
        y: cmp::min(rect.y1, rect.y2) as _,
        width: (rect.x2 - rect.x1).abs() as _,
        height: (rect.y2 - rect.y1).abs() as _,
    }
}
