use geo::{Coordinate, MultiPolygon};
use std::fmt;

/// A rectangular region with square edges.
#[derive(Debug, Clone)]
pub struct Rect<U = super::Unit> {
    rect: geo::Rect<f64>,
    inner: U,
}

impl Rect {
    /// Constructs a new rectangle using the provided corners.
    pub fn new(top_left: Coordinate<f64>, bottom_right: Coordinate<f64>) -> Self {
        Self {
            rect: geo::Rect::new(top_left, bottom_right),
            inner: super::Unit,
        }
    }

    /// Constructs a new rectangle given a center point and sizes.
    pub fn with_center(center: Coordinate<f64>, width: f64, height: f64) -> Self {
        Self {
            rect: geo::Rect::new(
                center
                    + Coordinate {
                        x: -width / 2.,
                        y: -height / 2.,
                    },
                center
                    + Coordinate {
                        x: width / 2.,
                        y: height / 2.,
                    },
            ),
            inner: super::Unit,
        }
    }
}

impl<U: super::InnerFeature> Rect<U> {
    /// Constructs a rectangle surrounding the inner feature. The
    /// origin of the inner feature will match the centeroid of the
    /// rectangle.
    pub fn with_inner(
        mut inner: U,
        top_left: Coordinate<f64>,
        bottom_right: Coordinate<f64>,
    ) -> Self {
        let rect = geo::Rect::new(top_left, bottom_right);
        inner.translate(rect.center());

        Self { rect, inner }
    }
}

impl<U: super::InnerFeature> fmt::Display for Rect<U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "rect({:?}, {:?}, U = {})",
            self.rect.min(),
            self.rect.max(),
            self.inner
        )
    }
}

impl<U: super::InnerFeature> super::Feature for Rect<U> {
    fn name(&self) -> &'static str {
        "rect"
    }

    fn edge(&self) -> Option<MultiPolygon<f64>> {
        Some(self.rect.clone().to_polygon().into())
    }

    fn translate(&mut self, v: Coordinate<f64>) {
        use geo::algorithm::translate::Translate;
        self.rect.translate_inplace(v.x, v.y);
        self.inner.translate(v);
    }

    fn interior(&self) -> Vec<super::InnerAtom> {
        self.inner.atoms()
    }
}
