// MIT/Apache2 License

#![cfg(windows)]

use crate::{
    color::Color,
    fill::FillRule,
    geometry::{Angle, GeometricArc, Line, Point, Rectangle},
    surface::{Surface, SurfaceFeatures},
    util::DebugContainer,
};
use std::{
    array::IntoIter as ArrayIter,
    cmp,
    collections::hash_map::{Entry, HashMap},
};
use yaww::{
    brush::{Brush, BrushFunctions},
    color::Color as YawwColor,
    dc::Dc,
    pen::{Pen, PenFunctions, PenStyle},
    task::Task,
    Point as YawwPoint, SendsDirective,
};

const FEATURES: SurfaceFeatures = SurfaceFeatures { gradients: false };

/// Yaww GDI drawing surface. This uses GDI to render on surfaces, even if it is slower than OpenGL or Direct2D.
#[derive(Debug)]
pub struct YawwGdiSurface<'thread, S> {
    thread: &'thread S,
    dc: Dc,
    residual: Option<YawwGdiSurfaceResidual>,
}

#[derive(Debug)]
pub struct YawwGdiSurfaceResidual {
    pen: Option<Color>,
    brush: Option<Color>,
    clear_brush: Option<Brush>,
    width: usize,
    task_queue: DebugContainer<Vec<Task<yaww::Result<()>>>>,
    pens: HashMap<(Color, usize), Pen>,
    brushes: HashMap<Color, Brush>,
}

impl<'thread, S> YawwGdiSurface<'thread, S> {
    #[inline]
    pub fn from_residual(thread: &'thread S, dc: Dc, residual: YawwGdiSurfaceResidual) -> Self {
        Self {
            thread,
            dc,
            residual: Some(residual),
        }
    }

    #[inline]
    pub fn new(thread: &'thread S, dc: Dc) -> Self {
        Self::from_residual(
            thread,
            dc,
            YawwGdiSurfaceResidual {
                pen: None,
                brush: None,
                clear_brush: None,
                width: 0,
                task_queue: DebugContainer::new(vec![]),
                pens: HashMap::new(),
                brushes: HashMap::new(),
            },
        )
    }

    #[inline]
    pub fn into_residual(self) -> YawwGdiSurfaceResidual {
        let mut residual = self.residual.unwrap();
        residual.pen = None;
        residual.brush = None;
        residual.clear_brush = None;
        residual
    }

    #[inline]
    fn residual(&mut self) -> &mut YawwGdiSurfaceResidual {
        self.residual.as_mut().expect("Already dropped?!?!")
    }
}

impl<'thread, S: SendsDirective> YawwGdiSurface<'thread, S> {
    #[inline]
    fn get_pen_from_color(&mut self, color: Color) -> crate::Result<Pen> {
        let width = self.residual().width;
        match self.residual().pens.get(&(color, width)) {
            Some(o) => Ok(*o),
            None => {
                let (r, g, b, _) = color.clamp_u8();
                let color2 = YawwColor::from_rgb(r, g, b);
                let pen = self
                    .thread
                    .create_pen(PenStyle::Solid, width as _, color2)?
                    .wait()?;
                self.residual().pens.insert((color, width), pen);
                Ok(pen)
            }
        }
    }

    #[cfg(feature = "async")]
    #[inline]
    async fn get_pen_from_color_async(&mut self, color: Color) -> crate::Result<Pen> {
        let width = self.residual().width;
        match self.residual().pens.get(&(color, width)) {
            Some(o) => Ok(*o),
            None => {
                let (r, g, b, _) = color.clamp_u8();
                let color2 = YawwColor::from_rgb(r, g, b);
                let pen = self
                    .thread
                    .create_pen(PenStyle::Solid, width as _, color2)?
                    .await?;
                self.residual().pens.insert((color, width), pen);
                Ok(pen)
            }
        }
    }

    #[inline]
    fn submit(&mut self, draw: DrawType) -> crate::Result {
        match draw {
            DrawType::Stroke => {
                // clear the fill
                if let Some(cf) = self.residual().clear_brush.take() {
                    self.dc.select_object(self.thread, cf)?.wait()?;
                }

                // install the stroke
                if let Some(s) = self.residual().pen.clone() {
                    let pen = self.get_pen_from_color(s)?;

                    self.dc.select_object(self.thread, pen)?.wait()?;
                }
            }
            DrawType::Fill => {
                // replace the stroke with a color
                if let Some(f) = self.residual().brush.clone() {
                    self.dc
                        .select_object(self.thread, self.get_pen_from_color(f)?)?
                        .wait()?;
                    let brush = match self.residual().brushes.get(&f) {
                        Some(o) => *o,
                        None => {
                            let (r, g, b, _) = f.clamp_u8();
                            let color = YawwColor::from_rgb(r, g, b);
                            let brush = self.thread.create_solid_brush(color)?.wait()?;
                            self.residual().brushes.insert(f, brush);
                            brush
                        }
                    };
                    let old_brush = self.dc.select_object(self.thread, brush)?.wait()?;
                    if self.residual().clear_brush.is_none() {
                        self.residual().clear_brush = Some(old_brush);
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg(feature = "async")]
    #[inline]
    async fn submit_async(&mut self, draw: DrawType) -> crate::Result {
        match draw {
            DrawType::Stroke => {
                // clear the fill
                if let Some(cf) = self.residual().clear_brush.take() {
                    self.dc.select_object(self.thread, cf)?.await;
                }

                // install the stroke
                if let Some(s) = self.residual().pen.clone() {
                    let pen = self.get_pen_from_color_async(s).await?;

                    self.dc.select_object(self.thread, pen)?.await?;
                }
            }
            DrawType::Fill => {
                // replace the stroke with a color
                if let Some(f) = self.residual().brush.clone() {
                    self.dc
                        .select_object(self.thread, self.get_pen_from_color_async(f)?)?
                        .await?;
                    let brush = match self.residual().brushes.get(&f) {
                        Some(o) => *o,
                        None => {
                            let (r, g, b, _) = f.clamp_u8();
                            let color = YawwColor::from_rgb(r, g, b);
                            let brush = self.thread.create_solid_brush(color)?.await?;
                            self.residual().brushes.insert(f, brush);
                            brush
                        }
                    };
                    let old_brush = self.dc.select_object(self.thread, brush)?.await?;
                    if self.residual().clear_brush.is_none() {
                        self.residual().clear_brush = Some(old_brush);
                    }
                }
            }
        }

        Ok(())
    }

    #[inline]
    fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        let t = ArrayIter::new([
            self.dc.move_to(self.thread, x1, y1)?,
            self.dc.line_to(self.thread, x2, y2)?,
        ]);
        self.residual().task_queue.extend(t);
        Ok(())
    }

    #[inline]
    fn lines(&mut self, lines: &[Line]) -> crate::Result {
        self.residual().task_queue.reserve(lines.len() * 2);
        lines
            .iter()
            .copied()
            .try_for_each::<_, crate::Result>(|Line { x1, y1, x2, y2 }| {
                let t = ArrayIter::new([
                    self.dc.move_to(self.thread, x1, y1)?,
                    self.dc.line_to(self.thread, x2, y2)?,
                ]);
                self.residual().task_queue.extend(t);
                Ok(())
            })
    }

    #[inline]
    fn rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        let t = self.dc.rectangle(self.thread, x1, y1, x2, y2)?;
        self.residual().task_queue.push(t);
        Ok(())
    }

    #[inline]
    fn rectangles(&mut self, rects: &[Rectangle]) -> crate::Result {
        self.residual().task_queue.reserve(rects.len());
        rects
            .iter()
            .copied()
            .try_for_each::<_, crate::Result>(|Rectangle { x1, y1, x2, y2 }| {
                let t = self.dc.rectangle(self.thread, x1, y1, x2, y2)?;
                self.residual().task_queue.push(t);
                Ok(())
            })
    }

    #[inline]
    fn arc(
        &mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        start: Angle,
        end: Angle,
    ) -> crate::Result {
        let [asx, asy, aex, aey] = calc_posns(GeometricArc {
            x1,
            y1,
            x2,
            y2,
            start,
            end,
        });
        let t = self
            .dc
            .arc(self.thread, x1, y1, x2, y2, asx, asy, aex, aey)?;
        self.residual().task_queue.push(t);
        Ok(())
    }

    #[inline]
    fn arcs(&mut self, arcs: &[GeometricArc]) -> crate::Result {
        self.residual().task_queue.reserve(arcs.len());
        arcs.iter()
            .copied()
            .try_for_each::<_, crate::Result>(|arc| {
                let GeometricArc { x1, y1, x2, y2, .. } = arc;
                let [asx, asy, aex, aey] = calc_posns(arc);
                let t = self
                    .dc
                    .arc(self.thread, x1, y1, x2, y2, asx, asy, aex, aey)?;
                self.residual().task_queue.push(t);
                Ok(())
            })
    }

    #[inline]
    fn ellipse(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        let t = self.dc.ellipse(self.thread, x1, y1, x2, y2)?;
        self.residual().task_queue.push(t);
        Ok(())
    }

    #[inline]
    fn ellipses(&mut self, rects: &[Rectangle]) -> crate::Result {
        self.residual().task_queue.reserve(rects.len());
        rects
            .iter()
            .copied()
            .try_for_each::<_, crate::Result>(|Rectangle { x1, y1, x2, y2 }| {
                let t = self.dc.ellipse(self.thread, x1, y1, x2, y2)?;
                self.residual().task_queue.push(t);
                Ok(())
            })
    }

    #[inline]
    fn polygon(&mut self, pts: &[Point]) -> crate::Result {
        let points: Vec<YawwPoint> = pts
            .iter()
            .copied()
            .map(|Point { x, y }| YawwPoint { x, y })
            .collect();
        let t = self.dc.polygon(self.thread, points)?;
        self.residual().task_queue.push(t);
        Ok(())
    }
}

#[derive(Copy, Clone)]
enum DrawType {
    Stroke,
    Fill,
}

use DrawType::{Fill, Stroke};

impl<'thread, S: SendsDirective> Surface for YawwGdiSurface<'thread, S> {
    #[inline]
    fn features(&self) -> SurfaceFeatures {
        FEATURES
    }

    #[inline]
    fn set_stroke(&mut self, color: Color) -> crate::Result {
        self.residual().pen = Some(color);
        Ok(())
    }

    #[inline]
    fn set_fill(&mut self, fill: FillRule) -> crate::Result {
        match fill {
            FillRule::SolidColor(color) => {
                self.residual().brush = Some(color);
                Ok(())
            }
            _ => Err(crate::Error::NotSupported(crate::NSOpType::Gradients)),
        }
    }

    #[inline]
    fn set_line_width(&mut self, width: usize) -> crate::Result {
        self.residual().width = width;
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> crate::Result {
        self.residual()
            .task_queue
            .drain(..)
            .try_for_each::<_, crate::Result>(|t| {
                t.wait()?;
                Ok(())
            })
    }

    #[inline]
    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        self.submit(Stroke)?;
        self.line(x1, y1, x2, y2)
    }

    #[inline]
    fn draw_lines(&mut self, lines: &[Line]) -> crate::Result {
        self.submit(Stroke)?;
        self.lines(lines)
    }

    #[inline]
    fn draw_rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        self.submit(Stroke)?;
        self.rectangle(x1, y1, x2, y2)
    }

    #[inline]
    fn draw_rectangles(&mut self, rects: &[Rectangle]) -> crate::Result {
        self.submit(Stroke)?;
        self.rectangles(rects)
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
        self.submit(Stroke)?;
        self.arc(x1, y1, x2, y2, start, end)
    }

    #[inline]
    fn draw_arcs(&mut self, arcs: &[GeometricArc]) -> crate::Result {
        self.submit(Stroke)?;
        self.arcs(arcs)
    }

    #[inline]
    fn draw_ellipse(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        self.submit(Stroke)?;
        self.ellipse(x1, y1, x2, y2)
    }

    #[inline]
    fn draw_ellipses(&mut self, rects: &[Rectangle]) -> crate::Result {
        self.submit(Stroke)?;
        self.ellipses(rects)
    }

    #[inline]
    fn fill_polygon(&mut self, points: &[Point]) -> crate::Result {
        self.submit(Fill)?;
        self.polygon(points)
    }

    #[inline]
    fn fill_rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        self.submit(Fill)?;
        self.rectangle(x1, y1, x2, y2)
    }

    #[inline]
    fn fill_rectangles(&mut self, rects: &[Rectangle]) -> crate::Result {
        self.submit(Fill)?;
        self.rectangles(rects)
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
        self.submit(Fill)?;
        self.arc(x1, y1, x2, y2, start, end)
    }

    #[inline]
    fn fill_arcs(&mut self, arcs: &[GeometricArc]) -> crate::Result {
        self.submit(Fill)?;
        self.arcs(arcs)
    }

    #[inline]
    fn fill_ellipse(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        self.submit(Fill)?;
        self.ellipse(x1, y1, x2, y2)
    }

    #[inline]
    fn fill_ellipses(&mut self, rects: &[Rectangle]) -> crate::Result {
        self.submit(Fill)?;
        self.ellipses(rects)
    }
}

#[cfg(feature = "async")]
impl<'thread> AsyncSurface for YawwGdiSurface<'thread> {
    #[inline]
    fn features(&self) -> SurfaceFeatures {
        FEATURES
    }

    #[inline]
    fn set_stroke(&mut self, color: Color) -> crate::Result {
        self.residual().pen = Some(color);
        Ok(())
    }

    #[inline]
    fn set_fill(&mut self, fill: FillRule) -> crate::Result {
        match fill {
            FillRule::SolidColor(color) => {
                self.residual().brush = Some(color);
                Ok(())
            }
            _ => Err(crate::Error::NotSupported(crate::NSOpType::Gradients)),
        }
    }

    #[inline]
    fn set_line_width(&mut self, width: usize) -> crate::Result {
        self.residual().width = width;
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> crate::Result {
        self.residual()
            .task_queue
            .drain(..)
            .try_for_each::<_, crate::Result>(|t| {
                t.wait()?;
                Ok(())
            })
    }

    #[inline]
    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        self.submit(Stroke)?;
        self.line(x1, y1, x2, y2)
    }

    #[inline]
    fn draw_lines(&mut self, lines: &[Line]) -> crate::Result {
        self.submit(Stroke)?;
        self.lines(lines)
    }

    #[inline]
    fn draw_rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        self.submit(Stroke)?;
        self.rectangle(x1, y1, x2, y2)
    }

    #[inline]
    fn draw_rectangles(&mut self, rects: &[Rectangle]) -> crate::Result {
        self.submit(Stroke)?;
        self.rectangles(rects)
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
        self.submit(Stroke)?;
        self.arc(x1, y1, x2, y2, start, end)
    }

    #[inline]
    fn draw_arcs(&mut self, arcs: &[GeometricArc]) -> crate::Result {
        self.submit(Stroke)?;
        self.arcs(arcs)
    }

    #[inline]
    fn draw_ellipse(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        self.submit(Stroke)?;
        self.ellipse(x1, y1, x2, y2)
    }

    #[inline]
    fn draw_ellipses(&mut self, rects: &[Rectangle]) -> crate::Result {
        self.submit(Stroke)?;
        self.ellipses(rects)
    }

    #[inline]
    fn fill_polygon(&mut self, points: &[Point]) -> crate::Result {
        self.submit(Fill)?;
        self.polygon(points)
    }

    #[inline]
    fn fill_rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        self.submit(Fill)?;
        self.rectangle(x1, y1, x2, y2)
    }

    #[inline]
    fn fill_rectangles(&mut self, rects: &[Rectangle]) -> crate::Result {
        self.submit(Fill)?;
        self.rectangles(rects)
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
        self.submit(Fill)?;
        self.arc(x1, y1, x2, y2, start, end)
    }

    #[inline]
    fn fill_arcs(&mut self, arcs: &[GeometricArc]) -> crate::Result {
        self.submit(Fill)?;
        self.arcs(arcs)
    }

    #[inline]
    fn fill_ellipse(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result {
        self.submit(Fill)?;
        self.ellipse(x1, y1, x2, y2)
    }

    #[inline]
    fn fill_ellipses(&mut self, rects: &[Rectangle]) -> crate::Result {
        self.submit(Fill)?;
        self.ellipses(rects)
    }
}

#[inline]
fn calc_posns(arc: GeometricArc) -> [i32; 4] {
    let GeometricArc { start, end, .. } = arc;
    let x1 = cmp::min(arc.x1, arc.x2) as f32;
    let y1 = cmp::min(arc.y1, arc.y2) as f32;
    let x2 = cmp::max(arc.x1, arc.x2) as f32;
    let y2 = cmp::max(arc.y1, arc.y2) as f32;
    let rx = (x2 - x1) / 2.0;
    let ry = (y2 - y1) / 2.0;
    let cx = x1 + rx;
    let cy = x2 + ry;

    let mut calc_posn = move |degree: f32| {
        (
            (cx + degree.cos() * rx).ceil() as i32,
            (cy + degree.sin() * ry).ceil() as i32,
        )
    };

    let (asx, asy) = calc_posn(start.radians());
    let (aex, aey) = calc_posn(end.radians());
    [asx, asy, aex, aey]
}

impl YawwGdiSurfaceResidual {
    #[inline]
    pub fn free<S: SendsDirective>(mut self, gt: &S) -> crate::Result {
        // TODO
        Ok(())
    }
}
