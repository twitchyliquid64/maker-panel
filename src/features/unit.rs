use geo::{Coordinate, MultiPolygon};
use std::fmt;

/// A feature with no geometry.
#[derive(Debug, Clone)]
pub struct Unit;

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "()")
    }
}

impl super::Feature for Unit {
    fn name(&self) -> &'static str {
        "unit"
    }

    fn edge(&self) -> Option<MultiPolygon<f64>> {
        None
    }

    fn translate(&mut self, _v: Coordinate<f64>) {}

    fn interior(&self) -> Vec<super::InnerAtom> {
        vec![]
    }
}

impl super::InnerFeature for Unit {
    fn name(&self) -> &'static str {
        "unit"
    }

    fn translate(&mut self, _v: Coordinate<f64>) {}

    fn atoms(&self) -> Vec<super::InnerAtom> {
        vec![]
    }
}
