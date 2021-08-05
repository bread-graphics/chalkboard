// MIT/Apache2 License

use lyon_geom::{Point, Vector};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Ellipse {
    pub center: Point,
    pub radii: Vector,
}
