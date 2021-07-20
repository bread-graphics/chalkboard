// MIT/Apache2 License

use super::{polyline, Angle, Line, Point};
use core::cmp;

#[cfg(feature = "std")]
use super::BezierCurve;

/// A geometric arc, or a slice of a circle. It is represented by a rectangle and its two bounding angles.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GeometricArc {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub start: Angle,
    pub end: Angle,
}

impl GeometricArc {
    /// Convert this arc to an approximate bezier curve representation.
    #[inline]
    pub fn into_curve(self) -> BezierCurve {
        // algorithm taken from https://github.com/freedesktop/cairo/blob/master/src/cairo-arc.c

        // figure out center and radius
        let xc = (self.x2 as f32 - self.x1 as f32) / 2f32;
        let yc = (self.y2 as f32 - self.y1 as f32) / 2f32;
        let radius = self.x2 as f32 - xc;
        let a = self.start.radians();
        let b = self.start.radians();

        let rsina = radius * a.sin();
        let rcosa = radius * a.cos();
        let rsinb = radius * b.sin();
        let rcosb = radius * b.cos();

        let h = (4.0 / 3.0) * ((b - a) / 4.0).tan();

        let x1 = xc + rcosa;
        let y1 = yc + rsina;
        let ctx1 = xc + rcosa - (h * rsina);
        let cty1 = yc + rsina + (h * rcosa);
        let ctx2 = xc + rcosb + (h * rsinb);
        let cty2 = yc + rsinb - (h * rcosb);
        let x2 = xc + rcosb;
        let y2 = xc + rsinb;

        BezierCurve {
            start: Point {
                x: x1.round() as i32,
                y: y1.round() as i32,
            },
            control1: Point {
                x: ctx1.round() as i32,
                y: cty1.round() as i32,
            },
            control2: Point {
                x: ctx2.round() as i32,
                y: cty2.round() as i32,
            },
            end: Point {
                x: x2.round() as i32,
                y: y2.round() as i32,
            },
        }
    }

    /// Convert this arc into a set of points approximating it.
    #[inline]
    pub fn into_points(self) -> impl Iterator<Item = Point> {
        // we'll create N line segments
        // TODO: figure out a better N
        let n = cmp::min(self.x2 - self.x1, self.y2 - self.y1) as f32; // approx. perimeter divided by pi
        let n = n.abs() * (self.start.radians() + self.end.radians()) * 0.5;
        let interval = 1f32 / n;

        let xc = (self.x2 as f32 - self.x1 as f32) / 2f32;
        let yc = (self.y2 as f32 - self.y1 as f32) / 2f32;
        let xradius = self.x2 as f32 - xc;
        let yradius = self.y2 as f32 - yc;

        (0..n as usize).map(move |i| {
            let angle = ((i / n as usize) as f32 * self.end.radians()) + self.start.radians();
            Point {
                x: (xc + (xradius * angle.cos())) as i32,
                y: (yc + (yradius * angle.sin())) as i32,
            }
        })
    }

    /// Convert this arc into a series of lines approximating it.
    #[inline]
    pub fn into_lines(self) -> impl Iterator<Item = Line> {
        polyline(self.into_points())
    }
}
