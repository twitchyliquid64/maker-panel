use geo::{Coordinate, MultiPolygon};
use std::fmt;

/// A feature which contains another but identifies the contained feature by name.
#[derive(Debug, Clone)]
pub struct Named<U = super::Unit> {
    feature: U,
    name: String,
}

impl<U: super::Feature + fmt::Debug + Clone> Named<U> {
    pub fn new(name: String, feature: U) -> Self {
        Self { feature, name }
    }
}

impl<U> fmt::Display for Named<U>
where
    U: super::Feature + fmt::Debug + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Named({:?}, {:?})", self.name, self.feature)
    }
}

impl<U> super::Feature for Named<U>
where
    U: super::Feature + fmt::Debug + Clone,
{
    fn name(&self) -> &'static str {
        "named"
    }

    fn edge_union(&self) -> Option<MultiPolygon<f64>> {
        self.feature.edge_union()
    }

    fn edge_subtract(&self) -> Option<MultiPolygon<f64>> {
        self.feature.edge_subtract()
    }

    fn translate(&mut self, v: Coordinate<f64>) {
        self.feature.translate(v)
    }

    fn interior(&self) -> Vec<super::InnerAtom> {
        self.feature.interior()
    }

    fn named_info(&self) -> Vec<super::NamedInfo> {
        use geo::algorithm::bounding_rect::BoundingRect;

        let union = self.feature.edge_union();

        let geometry = if union.is_none() {
            self.feature.edge_subtract()
        } else {
            union
        };

        let bounds = match geometry {
            Some(geometry) => geometry.bounding_rect().unwrap(),
            _ => geo::Rect::new((0., 0.), (0., 0.)),
        };

        vec![super::NamedInfo::new(self.name.clone(), bounds)]
    }
}
