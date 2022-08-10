use geo::{Coordinate, MultiPolygon};
use std::fmt;

/// A feature which is the origin-centered rotation of its contained geometry.
#[derive(Debug, Clone)]
pub struct Rotate<U = super::Unit> {
    features: Vec<U>,
    rotate: f64,
}

impl<U: super::Feature + fmt::Debug + Clone> Rotate<U> {
    pub fn new(rotate: f64, features: Vec<U>) -> Self {
        Self { features, rotate }
    }
}

impl<U> fmt::Display for Rotate<U>
where
    U: super::Feature + fmt::Debug + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Rotate({:?}, {:?})", self.rotate, self.features)
    }
}

impl<U> super::Feature for Rotate<U>
where
    U: super::Feature + fmt::Debug + Clone,
{
    fn name(&self) -> &'static str {
        "rotate"
    }

    fn edge_union(&self) -> Option<MultiPolygon<f64>> {
        use geo::algorithm::rotate::Rotate;

        self.features
            .iter()
            .map(|f| match f.edge_union() {
                Some(edge_geo) => Some(edge_geo.clone()),
                None => None,
            })
            .filter(|f| f.is_some())
            .map(|f| f.unwrap().rotate(self.rotate))
            .fold(None, |mut acc, g| {
                use geo_booleanop::boolean::BooleanOp;
                if let Some(current) = acc {
                    acc = Some(g.union(&current));
                } else {
                    acc = Some(g);
                };
                acc
            })
    }

    fn edge_subtract(&self) -> Option<MultiPolygon<f64>> {
        use geo::algorithm::rotate::Rotate;

        self.features
            .iter()
            .map(|f| match f.edge_subtract() {
                Some(edge_geo) => Some(edge_geo.clone()),
                None => None,
            })
            .filter(|f| f.is_some())
            .map(|f| f.unwrap().rotate(self.rotate))
            .fold(None, |mut acc, g| {
                use geo_booleanop::boolean::BooleanOp;
                if let Some(current) = acc {
                    acc = Some(g.union(&current));
                } else {
                    acc = Some(g);
                };
                acc
            })
    }

    fn translate(&mut self, v: Coordinate<f64>) {
        for e in self.features.iter_mut() {
            e.translate(v);
        }
    }

    fn interior(&self) -> Vec<super::InnerAtom> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::{Feature, Rect};

    #[test]
    fn identity() {
        let a = Rotate::new(0.0, vec![Rect::with_center([0., 0.].into(), 2., 3.)]);

        assert_eq!(a.edge_subtract(), None,);

        assert_eq!(
            a.edge_union(),
            Some(geo::MultiPolygon::from(vec![geo::Polygon::new(
                geo::LineString(vec![
                    geo::Coordinate { x: -1.0, y: -1.5 },
                    geo::Coordinate { x: -1.0, y: 1.5 },
                    geo::Coordinate { x: 1.0, y: 1.5 },
                    geo::Coordinate { x: 1.0, y: -1.5 },
                ]),
                vec![],
            )])),
        );
    }

    #[test]
    fn basic() {
        let a = Rotate::new(90.0, vec![Rect::with_center([0., 0.].into(), 1., 3.)]);

        use geo::bounding_rect::BoundingRect;
        let bounds = a.edge_union().unwrap().bounding_rect().unwrap();
        assert!(bounds.width() > 2.99);
        assert!(bounds.height() < 1.01);
    }
}
