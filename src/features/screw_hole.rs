use super::InnerAtom;
use crate::Layer;
use geo::Coordinate;
use std::fmt;

/// An interior feature representing a receptical for a fastener.
#[derive(Debug, Clone)]
pub struct ScrewHole {
    center: Coordinate<f64>,
    drill_radius: f64,
    annular_ring_radius: f64,
}

impl ScrewHole {
    /// Creates a screw hole with the specified diameter.
    pub fn with_diameter(dia: f64) -> Self {
        Self {
            drill_radius: dia / 2.0,
            annular_ring_radius: (dia / 2.0) + 0.3,
            ..Self::default()
        }
    }
}

impl Default for ScrewHole {
    fn default() -> Self {
        Self {
            center: [0., 0.].into(),
            drill_radius: 1.5,
            annular_ring_radius: 1.8,
        }
    }
}

impl fmt::Display for ScrewHole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "drill(center = {:?}, {}/{})",
            self.center, self.drill_radius, self.annular_ring_radius
        )
    }
}

impl super::InnerFeature for ScrewHole {
    fn name(&self) -> &'static str {
        "screw_hole"
    }

    fn translate(&mut self, v: Coordinate<f64>) {
        self.center = self.center + v;
    }

    fn atoms(&self) -> Vec<InnerAtom> {
        vec![
            InnerAtom::Circle {
                center: self.center,
                radius: self.annular_ring_radius,
                layer: Layer::BackCopper,
            },
            InnerAtom::Circle {
                center: self.center,
                radius: self.annular_ring_radius,
                layer: Layer::BackMask,
            },
            InnerAtom::Circle {
                center: self.center,
                radius: self.annular_ring_radius,
                layer: Layer::FrontCopper,
            },
            InnerAtom::Circle {
                center: self.center,
                radius: self.annular_ring_radius,
                layer: Layer::FrontMask,
            },
            InnerAtom::Drill {
                center: self.center,
                radius: self.drill_radius,
            },
        ]
    }
}
