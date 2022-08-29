use geo::{Coordinate, MultiPolygon};
use std::fmt;

/// A feature which is the negative of its contained geometry.
#[derive(Debug, Clone)]
pub struct Negative<U = super::Unit> {
    features: Vec<U>,
}

impl<U: super::Feature + fmt::Debug + Clone> Negative<U> {
    pub fn new(features: Vec<U>) -> Self {
        Self { features }
    }
}

impl<U> fmt::Display for Negative<U>
where
    U: super::Feature + fmt::Debug + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Negative({:?})", self.features)
    }
}

impl<U> super::Feature for Negative<U>
where
    U: super::Feature + fmt::Debug + Clone,
{
    fn name(&self) -> &'static str {
        "negative"
    }

    fn edge_union(&self) -> Option<MultiPolygon<f64>> {
        self.features
            .iter()
            .map(|f| match f.edge_subtract() {
                Some(edge_geo) => Some(edge_geo.clone()),
                None => None,
            })
            .filter(|f| f.is_some())
            .map(|f| f.unwrap())
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
        self.features
            .iter()
            .map(|f| match f.edge_union() {
                Some(edge_geo) => Some(edge_geo.clone()),
                None => None,
            })
            .filter(|f| f.is_some())
            .map(|f| f.unwrap())
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

    /// named_info returns information about named geometry.
    fn named_info(&self) -> Vec<super::NamedInfo> {
        self.features.iter().fold(vec![], |mut acc, feature| {
            for info in feature.named_info() {
                acc.push(info);
            }
            acc
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::{Feature, Rect};

    #[test]
    fn not_union() {
        let a = Negative::new(vec![
            Rect::with_center([0., 0.].into(), 2., 3.),
            Rect::with_center([0., 0.].into(), 3., 2.),
        ]);

        assert_eq!(a.edge_union(), None,);
        assert!(a.edge_subtract().is_some());
    }

    #[test]
    fn basic() {
        let a = Negative::new(vec![
            Rect::with_center([0., 0.].into(), 1., 1.),
            Rect::with_center([0.5, 0.].into(), 1., 1.),
        ]);

        assert_eq!(
            a.edge_subtract(),
            Some(geo::MultiPolygon::from(vec![geo::Polygon::new(
                geo::LineString(vec![
                    geo::Coordinate { x: -0.5, y: -0.5 },
                    geo::Coordinate { x: 0.0, y: -0.5 },
                    geo::Coordinate { x: 0.5, y: -0.5 },
                    geo::Coordinate { x: 1.0, y: -0.5 },
                    geo::Coordinate { x: 1.0, y: 0.5 },
                    geo::Coordinate { x: 0.5, y: 0.5 },
                    geo::Coordinate { x: 0.0, y: 0.5 },
                    geo::Coordinate { x: -0.5, y: 0.5 },
                    geo::Coordinate { x: -0.5, y: -0.5 }
                ]),
                vec![],
            )])),
        );
    }
}
