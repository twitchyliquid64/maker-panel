use geo::{Coordinate, MultiPolygon};
use std::fmt;

/// A triangular region with sharp edges.
#[derive(Debug, Clone)]
pub struct Triangle<U = super::Unit> {
    right_angle: bool,
    triangle: geo::Triangle<f64>,
    inner: U,
}

impl Triangle {
    /// Constructs a new triangle using the provided corners.
    pub fn new(c1: Coordinate<f64>, c2: Coordinate<f64>, c3: Coordinate<f64>) -> Self {
        Self {
            right_angle: false,
            triangle: geo::Triangle(c1, c2, c3),
            inner: super::Unit,
        }
    }

    /// Constructs a right-angle triangle, centered at the origin and
    /// with the given width and height.
    pub fn right_angle(width: f64, height: f64) -> Self {
        Self {
            right_angle: true,
            triangle: geo::Triangle(
                Coordinate {
                    x: -width / 2.,
                    y: -height / 2.,
                },
                Coordinate {
                    x: -width / 2.,
                    y: height / 2.,
                },
                Coordinate {
                    x: width / 2.,
                    y: height / 2.,
                },
            ),
            inner: super::Unit,
        }
    }
}

impl<U: super::InnerFeature + Clone + std::fmt::Debug> Triangle<U> {
    /// Constructs a triangle surrounding the inner feature. The
    /// origin of the inner feature will match the centeroid of the
    /// triangle.
    pub fn with_inner(inner: U) -> Self {
        let c1: Coordinate<f64> = [-1f64, -1f64].into();
        let c2: Coordinate<f64> = [-1f64, 1f64].into();
        let c3: Coordinate<f64> = [1f64, 1f64].into();
        let triangle = geo::Triangle(c1, c2, c3);
        Self {
            right_angle: true,
            triangle,
            inner,
        }
    }

    /// Returns a new right-angle triangle around the provided center.
    pub fn dimensions(mut self, center: Coordinate<f64>, width: f64, height: f64) -> Self {
        let triangle = geo::Triangle(
            center
                + Coordinate {
                    x: -width / 2.,
                    y: -height / 2.,
                },
            center
                + Coordinate {
                    x: -width / 2.,
                    y: height / 2.,
                },
            center
                + Coordinate {
                    x: width / 2.,
                    y: height / 2.,
                },
        );
        self.inner.translate(center);
        Self {
            triangle,
            right_angle: true,
            inner: self.inner,
        }
    }

    /// Returns a new triangle using the provided points. The inner feature will
    /// be translated to the centeroid of the triangle.
    pub fn bounds(mut self, c1: Coordinate<f64>, c2: Coordinate<f64>, c3: Coordinate<f64>) -> Self {
        use geo::algorithm::centroid::Centroid;
        let triangle = geo::Triangle(c1, c2, c3);
        self.inner
            .translate(triangle.to_polygon().centroid().unwrap().into());
        Self {
            triangle,
            right_angle: false,
            inner: self.inner,
        }
    }
}

impl<U: super::InnerFeature> fmt::Display for Triangle<U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "triangle({:?}, U = {})",
            self.triangle.to_array(),
            self.inner
        )
    }
}

impl<U: super::InnerFeature + Clone + std::fmt::Debug> super::Feature for Triangle<U> {
    fn name(&self) -> &'static str {
        "triangle"
    }

    fn edge_union(&self) -> Option<MultiPolygon<f64>> {
        Some(self.triangle.clone().to_polygon().into())
    }

    fn translate(&mut self, v: Coordinate<f64>) {
        use geo::algorithm::translate::Translate;
        self.triangle.translate_inplace(v.x, v.y);
        self.inner.translate(v);
    }

    fn interior(&self) -> Vec<super::InnerAtom> {
        self.inner.atoms()
    }
}
