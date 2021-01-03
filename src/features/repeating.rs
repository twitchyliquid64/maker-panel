use geo::{Coordinate, MultiPolygon};
use std::fmt;

/// A feature which is repeated in a tiling fashion.
#[derive(Debug, Clone)]
pub struct Tile<U = super::Rect> {
    inner: U,
    direction: crate::Direction,
    amt: usize,
}

impl<U: super::Feature> Tile<U> {
    /// Constructs a new tiling feature.
    pub fn new(inner: U, direction: crate::Direction, amt: usize) -> Self {
        Self {
            inner,
            direction,
            amt,
        }
    }
}

impl<U: super::Feature> fmt::Display for Tile<U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "repeating::Tile<{}>({} = {})",
            self.inner, self.direction, self.amt
        )
    }
}

impl<U: super::Feature + Clone> super::Feature for Tile<U> {
    fn name(&self) -> &'static str {
        "repeating::Tile"
    }

    fn edge(&self) -> Option<MultiPolygon<f64>> {
        match self.inner.edge() {
            Some(edge_geo) => {
                let mut out = edge_geo.clone();
                use geo::{bounding_rect::BoundingRect, translate::Translate};
                let bounds = edge_geo.bounding_rect().unwrap();

                for i in 0..self.amt {
                    let mut next = edge_geo.clone();
                    let (x, y) = self.direction.offset(bounds);
                    next.translate_inplace(i as f64 * x, i as f64 * y);

                    use geo_booleanop::boolean::BooleanOp;
                    out = out.union(&next);
                }
                println!("tiling geo = {:?}", out);
                Some(out)
            }
            None => None,
        }
    }

    fn translate(&mut self, v: Coordinate<f64>) {
        self.inner.translate(v)
    }

    fn interior(&self) -> Vec<super::InnerAtom> {
        let inner = self.inner.interior();
        let mut out = Vec::with_capacity(inner.len() * self.amt);

        let bounds = match self.inner.edge() {
            Some(edge_geo) => {
                use geo::bounding_rect::BoundingRect;
                edge_geo.bounding_rect().unwrap()
            }
            None => {
                use geo::{bounding_rect::BoundingRect, Geometry, GeometryCollection};
                let bounds = Geometry::GeometryCollection(GeometryCollection(
                    inner.iter().map(|a| Geometry::Rect(a.bounds())).collect(),
                ));
                bounds.bounding_rect().unwrap()
            }
        };

        for i in 0..self.amt {
            for v in inner.iter() {
                let mut v = v.clone();
                let (x, y) = self.direction.offset(bounds);
                v.translate(i as f64 * x, i as f64 * y);
                out.push(v);
            }
        }
        out
    }
}
