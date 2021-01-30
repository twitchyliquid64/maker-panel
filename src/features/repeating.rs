use geo::{Coordinate, MultiPolygon};
use std::fmt;

/// A feature which is repeated in a tiling fashion.
#[derive(Debug, Clone)]
pub struct Tile<U = super::Rect> {
    inner: U,
    direction: crate::Direction,
    amt: usize,
    v_score: bool,
}

impl<U: super::Feature> Tile<U> {
    /// Constructs a new tiling feature.
    pub fn new(inner: U, direction: crate::Direction, amt: usize) -> Self {
        let v_score = false;
        Self {
            inner,
            direction,
            amt,
            v_score,
        }
    }

    /// Returns a new tiling feature with the given v-score setting.
    pub fn v_score(mut self, v_score: bool) -> Self {
        self.v_score = v_score;
        self
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

    fn edge_subtract(&self) -> Option<MultiPolygon<f64>> {
        match self.inner.edge_subtract() {
            Some(sub_geo) => {
                let mut out = sub_geo.clone();

                use geo::{bounding_rect::BoundingRect, translate::Translate};
                let bounds = match self.inner.edge_union() {
                    Some(edge_geo) => edge_geo.bounding_rect().unwrap(),
                    None => sub_geo.clone().bounding_rect().unwrap(),
                };

                for i in 0..self.amt {
                    let mut next = sub_geo.clone();
                    let (x, y) = self.direction.offset(bounds);
                    next.translate_inplace(i as f64 * x, i as f64 * y);

                    use geo_booleanop::boolean::BooleanOp;
                    out = out.union(&next);
                }
                Some(out)
            }

            None => None,
        }
    }

    fn edge_union(&self) -> Option<MultiPolygon<f64>> {
        match self.inner.edge_union() {
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

        let bounds = match self.inner.edge_union() {
            Some(edge_geo) => {
                use geo::bounding_rect::BoundingRect;
                edge_geo.bounding_rect().unwrap()
            }
            None => {
                use geo::{bounding_rect::BoundingRect, Geometry, GeometryCollection};
                let bounds = Geometry::GeometryCollection(GeometryCollection(
                    inner
                        .iter()
                        .map(|a| a.bounds())
                        .filter(|b| b.is_some())
                        .map(|b| Geometry::Rect(b.unwrap()))
                        .collect(),
                ));
                bounds.bounding_rect().unwrap()
            }
        };

        for i in 0..self.amt {
            let (x, y) = self.direction.offset(bounds);
            let (x, y) = (i as f64 * x, i as f64 * y);

            for v in inner.iter() {
                let mut v = v.clone();
                v.translate(x, y);
                out.push(v);
            }

            if self.v_score && i < self.amt - 1 {
                let (x, y) = (x + bounds.width() / 2., y + bounds.height() / 2.);

                out.push(match self.direction {
                    crate::Direction::Left | crate::Direction::Right => {
                        super::InnerAtom::VScoreV(x)
                    }
                    crate::Direction::Down | crate::Direction::Up => super::InnerAtom::VScoreH(y),
                })
            }
        }
        out
    }
}
