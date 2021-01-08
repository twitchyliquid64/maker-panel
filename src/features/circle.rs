use geo::{Coordinate, MultiPolygon, Point, Polygon};
use std::fmt;

/// A circular region.
#[derive(Debug, Clone)]
pub struct Circle<U = super::Unit> {
    center: Coordinate<f64>,
    radius: f64,
    inner: U,
}

impl Circle {
    /// Constructs a new circle using the provided center and radius.
    pub fn new(center: Coordinate<f64>, radius: f64) -> Self {
        Self {
            center,
            radius,
            inner: super::Unit,
        }
    }

    /// Constructs a new circle with the provided radius, centered on
    /// the origin.
    pub fn with_radius(radius: f64) -> Self {
        Self::new([0.0, 0.0].into(), radius)
    }
}

impl<U: super::InnerFeature + Clone> Circle<U> {
    /// Constructs a circle surrounding the inner feature. The
    /// origin of the inner feature will be positioned at the
    /// center of the circle.
    pub fn with_inner(mut inner: U, center: Coordinate<f64>, radius: f64) -> Self {
        inner.translate(center);

        Self {
            center,
            radius,
            inner,
        }
    }

    /// Constructs a circle with a given radius surrounding the inner feature.
    pub fn wrap_with_radius(inner: U, radius: f64) -> Self {
        Self {
            radius,
            inner,
            center: [0.0, 0.0].into(),
        }
    }
}

impl<U: super::InnerFeature> fmt::Display for Circle<U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "circle({:?}, r = {:?}, U = {})",
            self.center, self.radius, self.inner
        )
    }
}

impl<U: super::InnerFeature + Clone + std::fmt::Debug> super::Feature for Circle<U> {
    fn name(&self) -> &'static str {
        "circle"
    }

    fn edge_union(&self) -> Option<MultiPolygon<f64>> {
        use geo::algorithm::rotate::RotatePoint;
        let right_edge: Point<_> = (self.center.x + self.radius, self.center.y).into();
        let mut out = Vec::with_capacity(361);

        for i in 0..=360 {
            out.push(right_edge.rotate_around_point(i as f64, self.center.into()));
        }

        Some(MultiPolygon(vec![Polygon::new(
            geo::LineString(out.into_iter().map(|p| p.into()).collect()),
            vec![],
        )]))
    }

    fn translate(&mut self, v: Coordinate<f64>) {
        self.center = self.center + v;
        self.inner.translate(v);
    }

    fn interior(&self) -> Vec<super::InnerAtom> {
        self.inner.atoms()
    }
}
