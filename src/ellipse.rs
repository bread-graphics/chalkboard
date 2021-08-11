// MIT/Apache2 License

use lyon_geom::{Point, Vector};

#[derive(Debug, Copy, Clone)]
pub struct Ellipse {
    pub center: Point<f32>,
    pub radii: Vector<f32>,
}
