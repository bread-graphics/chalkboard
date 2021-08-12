// MIT/Apache2 License

//! Fallback BreadX software renderer. This uses the native xproto commands for drawing. Note that these are
//! usually slower than their XRender or GLX equivalents.

use super::image;
use crate::{
    fill::FillRule,
    surface::{Surface, SurfaceFeatures},
    util::clamp,
    Color, Image, ImageFormat,
};
use breadx::{
    auto::xproto::{
        Arc as XArc, Colormap, CoordMode, GetGeometryReply, Pixmap, Point as XPoint, PolyShape,
        Rectangle as XRect, Segment, Window,
    },
    display::{prelude::*, Display, DisplayBase, GcParameters},
    Drawable, Gcontext,
};
use lyon_geom::{Angle, Arc, LineSegment, Point, Rect, Size, Vector};
use std::{
    cmp::Ordering,
    collections::hash_map::{Entry, HashMap},
    mem::{self, MaybeUninit},
    num::NonZeroUsize,
    ptr,
};

#[cfg(feature = "async")]
use breadx::display::AsyncConnection;

const FEATURES: SurfaceFeatures = SurfaceFeatures {
    transparency: false,
    gradients: false,
    floats: false,
};

/// Fallback BreadX surface. This uses XProto commands to render, even if they are slower than XRender or OpenGL
/// rendering.
#[derive(Debug)]
pub struct FallbackBreadxSurface<'dpy, Dpy: DisplayBase + ?Sized> {
    // display
    display: &'dpy mut Dpy,
    old_checked: bool,

    // window
    target: Drawable,
    gc: Gcontext,
    cmap: Colormap,

    // color management
    // note: colormapper is guaranteed to be Some unless into_map is called
    mapper: Option<ColorMapper>,
    manager: ColorManager,

    line_width: Option<usize>,
}

/// Maps our colors to breadx pixel colors.
#[derive(Debug)]
struct ColorMapper {
    map: HashMap<Color, u32>,
}

impl ColorMapper {
    #[inline]
    fn new(map: HashMap<Color, u32>) -> Self {
        Self { map }
    }

    #[inline]
    fn map(self) -> HashMap<Color, u32> {
        self.map
    }

    #[inline]
    fn map_color<Dpy: Display + ?Sized>(
        &mut self,
        dpy: &mut Dpy,
        cmap: Colormap,
        color: Color,
    ) -> crate::Result<u32> {
        match self.map.entry(color) {
            Entry::Occupied(o) => Ok(*o.get()),
            Entry::Vacant(v) => {
                let r: u16 = clamp(color.red());
                let g: u16 = clamp(color.green());
                let b: u16 = clamp(color.blue());
                let clr = cmap.alloc_color_immediate(dpy, r, g, b)?.pixel();
                Ok(*v.insert(clr))
            }
        }
    }

    #[cfg(feature = "async")]
    #[inline]
    async fn map_color_async<Dpy: AsyncDisplay + ?Sized>(
        &mut self,
        dpy: &mut Dpy,
        cmap: Colormap,
        color: Color,
    ) -> crate::Result<u32> {
        match self.map.entry(color) {
            Entry::Occupied(o) => Ok(*o.get()),
            Entry::Vacant(v) => {
                let r: u16 = clamp(color.red());
                let g: u16 = clamp(color.green());
                let b: u16 = clamp(color.blue());
                let clr = cmap
                    .alloc_color_immediate_async(dpy, r, g, b)
                    .await?
                    .pixel();
                Ok(*v.insert(clr))
            }
        }
    }
}

/// Figure out which color to set to the drawing color of the GC.
#[derive(Debug, Default)]
struct ColorManager {
    stroke: ManagedColor,
    fill: ManagedColor,
}

#[derive(Debug, Copy, Clone)]
enum ManagedColor {
    Submitted(u32),
    Unsubmitted(u32),
}

impl Default for ManagedColor {
    #[inline]
    fn default() -> Self {
        Self::Submitted(0)
    }
}

#[derive(Copy, Clone)]
enum DrawType {
    Stroke,
    Fill,
}

use DrawType::{Fill, Stroke};

impl ColorManager {
    #[inline]
    fn set_stroke(&mut self, clr: u32) {
        self.stroke = ManagedColor::Unsubmitted(clr);
    }

    #[inline]
    fn set_fill(&mut self, clr: u32) {
        self.fill = ManagedColor::Unsubmitted(clr);
    }

    #[inline]
    fn submit_stroke(&mut self) -> Option<u32> {
        match self.stroke {
            ManagedColor::Submitted(_) => None,
            ManagedColor::Unsubmitted(stroke) => {
                if let ManagedColor::Submitted(fill) = self.fill {
                    self.fill = ManagedColor::Unsubmitted(fill);
                }

                Some(stroke)
            }
        }
    }

    #[inline]
    fn submit_fill(&mut self) -> Option<u32> {
        match self.fill {
            ManagedColor::Submitted(_) => None,
            ManagedColor::Unsubmitted(fill) => {
                if let ManagedColor::Submitted(stroke) = self.stroke {
                    self.stroke = ManagedColor::Unsubmitted(stroke);
                }

                Some(fill)
            }
        }
    }
}

impl<'dpy, Dpy: DisplayBase + ?Sized> FallbackBreadxSurface<'dpy, Dpy> {
    #[inline]
    fn mapper(&mut self) -> &mut ColorMapper {
        self.mapper.as_mut().expect("NPP")
    }

    /// Construct a new instance of a FallbackBreadxSurface.
    #[inline]
    pub fn new<Target: Into<Drawable>>(dpy: &'dpy mut Dpy, target: Target, gc: Gcontext) -> Self {
        Self::with_cached_colormap(dpy, target, gc, HashMap::new())
    }

    /// Destroy this surface and get the cached color map from its remains.
    #[inline]
    pub fn into_colormap(mut self) -> HashMap<Color, u32> {
        self.mapper.take().expect("NPP").map()
    }

    /// Create a new surface from a cached color map. This can speed up certain computations.
    #[inline]
    pub fn with_cached_colormap<Target: Into<Drawable>>(
        dpy: &'dpy mut Dpy,
        target: Target,
        gc: Gcontext,
        map: HashMap<Color, u32>,
    ) -> Self {
        let cmap = dpy.default_colormap();
        let old_checked = dpy.checked();
        dpy.set_checked(false);
        Self {
            old_checked,
            display: dpy,
            target: target.into(),
            gc,
            cmap,
            mapper: Some(ColorMapper::new(map)),
            manager: Default::default(),
            line_width: None,
        }
    }

    #[inline]
    fn submit_draw_params(&mut self, draw_type: DrawType) -> Option<GcParameters> {
        let mut changed = false;
        let mut params = GcParameters::default();

        if let Some(d) = match draw_type {
            Stroke => self.manager.submit_stroke(),
            Fill => self.manager.submit_fill(),
        } {
            changed = true;
            params.foreground = Some(d);
        }

        if let Some(line_width) = self.line_width.take() {
            params.line_width = Some(line_width as _);
        }

        if changed {
            Some(params)
        } else {
            None
        }
    }
}

impl<'dpy, Dpy: DisplayBase + ?Sized> Drop for FallbackBreadxSurface<'dpy, Dpy> {
    #[inline]
    fn drop(&mut self) {
        self.display.set_checked(self.old_checked);
    }
}

impl<'dpy, Dpy: Display + ?Sized> FallbackBreadxSurface<'dpy, Dpy> {
    #[inline]
    fn submit_draw(&mut self, draw_type: DrawType) -> crate::Result {
        if let Some(params) = self.submit_draw_params(draw_type) {
            self.gc.change(self.display, params)?;
        }

        Ok(())
    }
}

#[cfg(feature = "async")]
impl<'dpy, Dpy: AsyncDisplay + ?Sized> FallbackBreadxSurface<'dpy, Dpy> {
    #[inline]
    async fn submit_draw_async(&mut self, draw_type: DrawType) -> crate::Result {
        if let Some(params) = self.submit_draw_params(draw_type) {
            self.gc.change_async(self.display, params).await?;
        }

        Ok(())
    }
}

impl<'dpy, Dpy: Display + ?Sized> Surface for FallbackBreadxSurface<'dpy, Dpy> {
    #[inline]
    fn features(&self) -> SurfaceFeatures {
        FEATURES
    }

    #[inline]
    fn set_stroke(&mut self, color: Color) -> crate::Result {
        let clr = self
            .mapper
            .as_mut()
            .unwrap()
            .map_color(self.display, self.cmap, color)?;
        self.manager.set_stroke(clr);
        Ok(())
    }

    #[inline]
    fn set_fill(&mut self, rule: FillRule) -> crate::Result {
        if let FillRule::SolidColor(color) = rule {
            let clr = self
                .mapper
                .as_mut()
                .unwrap()
                .map_color(self.display, self.cmap, color)?;
            self.manager.set_fill(clr);
            Ok(())
        } else {
            Err(crate::Error::NotSupported(crate::NSOpType::Gradients))
        }
    }

    #[inline]
    fn set_line_width(&mut self, width: usize) -> crate::Result {
        self.line_width = Some(width);
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> crate::Result {
        self.display.synchronize()?;
        Ok(())
    }

    #[inline]
    fn create_image(
        &mut self,
        image_bytes: &[u8],
        width: u32,
        height: u32,
        image_format: ImageFormat,
    ) -> crate::Result<Image> {
        let target = self.target;
        let pixmap = image::image_to_pixmap(
            &mut self.display,
            target,
            image_bytes,
            width,
            height,
            image_format,
        )?;

        Ok(Image::from_raw(
            NonZeroUsize::new(pixmap.xid as usize).expect("Pixmap should never be zero"),
        ))
    }

    #[inline]
    fn destroy_image(&mut self, image: Image) -> crate::Result {
        let pixmap = Pixmap::const_from_xid(image.into_raw().get() as u32);
        pixmap.free(self.display)?;
        Ok(())
    }

    #[inline]
    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> crate::Result {
        self.submit_draw(Stroke)?;
        self.gc.draw_line(
            self.display,
            self.target,
            Segment {
                x1: x1 as _,
                y1: y1 as _,
                x2: x2 as _,
                y2: y2 as _,
            },
        )?;
        Ok(())
    }

    #[inline]
    fn draw_lines(&mut self, lines: &[LineSegment<f32>]) -> crate::Result {
        self.submit_draw(Stroke)?;
        let lines: Vec<Segment> = lines
            .iter()
            .copied()
            .map(
                |LineSegment {
                     from: Point { x: x1, y: y1, .. },
                     to: Point { x: x2, y: y2, .. },
                 }| Segment {
                    x1: x1 as _,
                    y1: y1 as _,
                    x2: x2 as _,
                    y2: y2 as _,
                },
            )
            .collect();
        self.gc.draw_lines(self.display, self.target, &lines)?;
        Ok(())
    }

    #[inline]
    fn draw_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32) -> crate::Result {
        self.submit_draw(Stroke)?;
        let rect = XRect {
            x: x as _,
            y: y as _,
            width: width as _,
            height: height as _,
        };
        self.gc.draw_rectangle(self.display, self.target, rect)?;
        Ok(())
    }

    #[inline]
    fn draw_rectangles(&mut self, rects: &[Rect<f32>]) -> crate::Result {
        self.submit_draw(Stroke)?;
        let rects: Vec<XRect> = rects
            .iter()
            .copied()
            .map(
                |Rect {
                     origin: Point { x, y, .. },
                     size: Size { width, height, .. },
                 }| XRect {
                    x: x as _,
                    y: y as _,
                    width: width as _,
                    height: height as _,
                },
            )
            .collect();
        self.gc.draw_rectangles(self.display, self.target, rects)?;
        Ok(())
    }

    #[inline]
    fn draw_arc(
        &mut self,
        xcenter: f32,
        ycenter: f32,
        xradius: f32,
        yradius: f32,
        start_angle: Angle<f32>,
        sweep_angle: Angle<f32>,
    ) -> crate::Result {
        self.submit_draw(Stroke)?;
        let arc = convert_arc(
            xcenter as _,
            ycenter as _,
            xradius as _,
            yradius as _,
            start_angle,
            sweep_angle,
        );
        self.gc.draw_arc(self.display, self.target, arc)?;
        Ok(())
    }

    #[inline]
    fn draw_arcs(&mut self, arcs: &[Arc<f32>]) -> crate::Result {
        self.submit_draw(Stroke)?;
        let arcs: Vec<XArc> = arcs
            .iter()
            .copied()
            .map(
                |Arc {
                     center:
                         Point {
                             x: xcenter,
                             y: ycenter,
                             ..
                         },
                     radii:
                         Vector {
                             x: xradius,
                             y: yradius,
                             ..
                         },
                     start_angle,
                     sweep_angle,
                     ..
                 }| {
                    convert_arc(
                        xcenter as _,
                        ycenter as _,
                        xradius as _,
                        yradius as _,
                        start_angle,
                        sweep_angle,
                    )
                },
            )
            .collect();
        self.gc.draw_arcs(self.display, self.target, arcs)?;
        Ok(())
    }

    #[inline]
    fn fill_polygon(&mut self, points: &[Point<f32>]) -> crate::Result {
        self.submit_draw(Fill)?;
        let points: Vec<XPoint> = points
            .iter()
            .copied()
            .map(|Point { x, y, .. }| XPoint {
                x: x as _,
                y: y as _,
            })
            .collect();
        self.gc.fill_polygon(
            self.display,
            self.target,
            PolyShape::Complex,
            CoordMode::Origin,
            points,
        )?;
        Ok(())
    }

    #[inline]
    fn fill_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32) -> crate::Result {
        self.submit_draw(Fill)?;
        let rect = XRect {
            x: x as _,
            y: y as _,
            width: width as _,
            height: height as _,
        };
        self.gc.fill_rectangle(self.display, self.target, rect)?;
        Ok(())
    }

    #[inline]
    fn fill_rectangles(&mut self, rects: &[Rect<f32>]) -> crate::Result {
        self.submit_draw(Fill)?;
        let rects: Vec<XRect> = rects
            .iter()
            .copied()
            .map(
                |Rect {
                     origin: Point { x, y, .. },
                     size: Size { width, height, .. },
                 }| XRect {
                    x: x as _,
                    y: y as _,
                    width: width as _,
                    height: height as _,
                },
            )
            .collect();
        self.gc.fill_rectangles(self.display, self.target, rects)?;
        Ok(())
    }

    #[inline]
    fn fill_arc(
        &mut self,
        xcenter: f32,
        ycenter: f32,
        xradius: f32,
        yradius: f32,
        start_angle: Angle<f32>,
        sweep_angle: Angle<f32>,
    ) -> crate::Result {
        self.submit_draw(Fill)?;
        let arc = convert_arc(
            xcenter as _,
            ycenter as _,
            xradius as _,
            yradius as _,
            start_angle,
            sweep_angle,
        );
        self.gc.fill_arc(self.display, self.target, arc)?;
        Ok(())
    }

    #[inline]
    fn fill_arcs(&mut self, arcs: &[Arc<f32>]) -> crate::Result {
        self.submit_draw(Fill)?;
        let arcs: Vec<XArc> = arcs
            .iter()
            .copied()
            .map(
                |Arc {
                     center:
                         Point {
                             x: xcenter,
                             y: ycenter,
                             ..
                         },
                     radii:
                         Vector {
                             x: xradius,
                             y: yradius,
                             ..
                         },
                     start_angle,
                     sweep_angle,
                     ..
                 }| {
                    convert_arc(
                        xcenter as _,
                        ycenter as _,
                        xradius as _,
                        yradius as _,
                        start_angle,
                        sweep_angle,
                    )
                },
            )
            .collect();
        self.gc.fill_arcs(self.display, self.target, arcs)?;
        Ok(())
    }

    #[inline]
    fn copy_image(
        &mut self,
        src: Image,
        src_x: i32,
        src_y: i32,
        dst_x: i32,
        dst_y: i32,
        width: u32,
        height: u32,
    ) -> crate::Result {
        let pixmap = Pixmap::const_from_xid(src.into_raw().get() as u32);
        self.display.copy_area(
            pixmap,
            self.target,
            self.gc,
            src_x as _,
            src_y as _,
            width as _,
            height as _,
            dst_x as _,
            dst_y as _,
        )?;
        Ok(())
    }
}

#[inline]
fn convert_arc(
    xcenter: i32,
    ycenter: i32,
    xradius: i32,
    yradius: i32,
    start: Angle<f32>,
    end: Angle<f32>,
) -> XArc {
    let x = xcenter - xradius;
    let y = ycenter - yradius;
    let width = xradius * 2;
    let height = yradius * 2;

    XArc {
        x: x as i16,
        y: y as i16,
        width: width as u16,
        height: height as u16,
        angle1: convert_angle(start),
        angle2: convert_angle(end),
    }
}

#[inline]
fn convert_angle(angle: Angle<f32>) -> i16 {
    let angle = angle
        + Angle {
            radians: std::f32::consts::FRAC_PI_4,
        };
    (angle.to_degrees() * 64.0) as i16
}
