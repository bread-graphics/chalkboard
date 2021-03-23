// MIT/Apache2 License

//! Fallback BreadX software renderer. This uses the native xproto commands for drawing. Note that these are
//! usually slower than their XRender or GLX equivalents.

use crate::{
    color::Color,
    fill::FillRule,
    geometry::{Angle, GeometricArc, Line, Point, Rectangle},
    surface::{Surface, SurfaceFeatures},
    util::clamp,
};
use breadx::{
    auto::xproto::{
        Arc as XArc, Colormap, CoordMode, Point as XPoint, PolyShape, Rectangle as XRect, Segment,
    },
    display::{Connection, Display, GcParameters},
    Drawable, Gcontext,
};
use std::{
    cmp::Ordering,
    collections::hash_map::{Entry, HashMap},
    mem::{self, MaybeUninit},
    ptr,
};

#[cfg(feature = "async")]
use breadx::display::AsyncConnection;

const FEATURES: SurfaceFeatures = SurfaceFeatures { gradients: false };

/// Fallback BreadX surface. This uses XProto commands to render, even if they are slower than XRender or OpenGL
/// rendering.
#[derive(Debug)]
pub struct FallbackBreadxSurface<'dpy, Conn> {
    // display
    display: &'dpy mut Display<Conn>,
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
    fn map_color<Conn: Connection>(
        &mut self,
        dpy: &mut Display<Conn>,
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
    async fn map_color_async<Conn: AsyncConnection + Send>(
        &mut self,
        dpy: &mut Display<Conn>,
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

impl<'dpy, Conn> FallbackBreadxSurface<'dpy, Conn> {
    #[inline]
    fn mapper(&mut self) -> &mut ColorMapper {
        self.mapper.as_mut().expect("NPP")
    }

    /// Construct a new instance of a FallbackBreadxSurface.
    #[inline]
    pub fn new<Target: Into<Drawable>>(
        dpy: &'dpy mut Display<Conn>,
        target: Target,
        gc: Gcontext,
    ) -> Self {
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
        dpy: &'dpy mut Display<Conn>,
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

impl<'dpy, Conn> Drop for FallbackBreadxSurface<'dpy, Conn> {
    #[inline]
    fn drop(&mut self) {
        self.display.set_checked(self.old_checked);
    }
}

impl<'dpy, Conn: Connection> FallbackBreadxSurface<'dpy, Conn> {
    #[inline]
    fn submit_draw(&mut self, draw_type: DrawType) -> crate::Result {
        if let Some(params) = self.submit_draw_params(draw_type) {
            self.gc.change(self.display, params)?;
        }

        Ok(())
    }
}

#[cfg(feature = "async")]
impl<'dpy, Conn: AsyncConnection + Send> FallbackBreadxSurface<'dpy, Conn> {
    #[inline]
    async fn submit_draw_async(&mut self, draw_type: DrawType) -> crate::Result {
        if let Some(params) = self.submit_draw_params(draw_type) {
            self.gc.change_async(self.display, params).await?;
        }

        Ok(())
    }
}

impl<'dpy, Conn: Connection> Surface for FallbackBreadxSurface<'dpy, Conn> {
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
    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
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
    fn draw_lines(&mut self, lines: &[Line]) -> crate::Result {
        self.submit_draw(Stroke)?;
        let lines: Vec<Segment> = lines
            .iter()
            .copied()
            .map(|Line { x1, y1, x2, y2 }| Segment {
                x1: x1 as _,
                y1: y1 as _,
                x2: x2 as _,
                y2: y2 as _,
            })
            .collect();
        self.gc.draw_lines(self.display, self.target, &lines)?;
        Ok(())
    }

    #[inline]
    fn draw_rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        self.submit_draw(Stroke)?;
        let rect = convert_rect(x1, y1, x2, y2);
        self.gc.draw_rectangle(self.display, self.target, rect)?;
        Ok(())
    }

    #[inline]
    fn draw_rectangles(&mut self, rects: &[Rectangle]) -> crate::Result {
        self.submit_draw(Stroke)?;
        let rects: Vec<XRect> = rects
            .iter()
            .copied()
            .map(|Rectangle { x1, y1, x2, y2 }| convert_rect(x1, y1, x2, y2))
            .collect();
        self.gc.draw_rectangles(self.display, self.target, &rects)?;
        Ok(())
    }

    #[inline]
    fn draw_arc(
        &mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        start: Angle,
        end: Angle,
    ) -> crate::Result {
        self.submit_draw(Stroke)?;
        let arc = convert_arc(x1, y1, x2, y2, start, end);
        self.gc.draw_arc(self.display, self.target, arc)?;
        Ok(())
    }

    #[inline]
    fn draw_arcs(&mut self, arcs: &[GeometricArc]) -> crate::Result {
        self.submit_draw(Stroke)?;
        let arcs: Vec<XArc> = arcs
            .iter()
            .copied()
            .map(
                |GeometricArc {
                     x1,
                     y1,
                     x2,
                     y2,
                     start,
                     end,
                 }| convert_arc(x1, y1, x2, y2, start, end),
            )
            .collect();
        self.gc.draw_arcs(self.display, self.target, &arcs)?;
        Ok(())
    }

    #[inline]
    fn fill_polygon(&mut self, points: &[Point]) -> crate::Result {
        self.submit_draw(Fill)?;
        let points: Vec<XPoint> = points
            .iter()
            .copied()
            .map(|Point { x, y }| XPoint {
                x: x as _,
                y: y as _,
            })
            .collect();
        self.gc.fill_polygon(
            self.display,
            self.target,
            PolyShape::Complex,
            CoordMode::Origin,
            &points,
        )?;
        Ok(())
    }

    #[inline]
    fn fill_rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        self.submit_draw(Fill)?;
        let rect = convert_rect(x1, y1, x2, y2);
        self.gc.fill_rectangle(self.display, self.target, rect)?;
        Ok(())
    }

    #[inline]
    fn fill_rectangles(&mut self, rects: &[Rectangle]) -> crate::Result {
        self.submit_draw(Fill)?;
        let rects: Vec<XRect> = rects
            .iter()
            .copied()
            .map(|Rectangle { x1, y1, x2, y2 }| convert_rect(x1, y1, x2, y2))
            .collect();
        self.gc.fill_rectangles(self.display, self.target, &rects)?;
        Ok(())
    }

    #[inline]
    fn fill_arc(
        &mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        start: Angle,
        end: Angle,
    ) -> crate::Result {
        self.submit_draw(Fill)?;
        let arc = convert_arc(x1, y1, x2, y2, start, end);
        self.gc.fill_arc(self.display, self.target, arc)?;
        Ok(())
    }

    #[inline]
    fn fill_arcs(&mut self, arcs: &[GeometricArc]) -> crate::Result {
        self.submit_draw(Fill)?;
        let arcs: Vec<XArc> = arcs
            .iter()
            .copied()
            .map(
                |GeometricArc {
                     x1,
                     y1,
                     x2,
                     y2,
                     start,
                     end,
                 }| convert_arc(x1, y1, x2, y2, start, end),
            )
            .collect();
        self.gc.fill_arcs(self.display, self.target, &arcs)?;
        Ok(())
    }
}

#[inline]
fn convert_rect(x1: i32, y1: i32, x2: i32, y2: i32) -> XRect {
    let (xl, xs) = if x1 <= x2 { (x2, x1) } else { (x1, x2) };
    let (yl, ys) = if y1 <= y2 { (y2, y1) } else { (y1, y2) };

    let width: u32 = (xl - xs) as u32;
    let height: u32 = (yl - ys) as u32;
    XRect {
        x: xs as _,
        y: ys as _,
        width: width as _,
        height: height as _,
    }
}

#[inline]
fn convert_arc(x1: i32, y1: i32, x2: i32, y2: i32, start: Angle, end: Angle) -> XArc {
    let XRect {
        x,
        y,
        width,
        height,
    } = convert_rect(x1, y1, x2, y2);
    XArc {
        x,
        y,
        width,
        height,
        angle1: convert_angle(start + Angle::QUARTER_CIRCLE),
        angle2: convert_angle(end + Angle::QUARTER_CIRCLE),
    }
}

#[inline]
fn convert_angle(angle: Angle) -> i16 {
    (angle.degrees() * 16.0) as i16
}
