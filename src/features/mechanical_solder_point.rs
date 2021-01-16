use super::InnerAtom;
use crate::Layer;
use geo::{Coordinate, Rect};
use std::fmt;

/// An interior feature that just renders a smiley on the front silkscreen.
#[derive(Debug, Clone)]
pub struct MechanicalSolderPoint {
    center: Coordinate<f64>,
    size: (f64, f64),
    drill_radius: f64,
}

impl MechanicalSolderPoint {
    /// Constructs an MSP with the specified width and height.
    pub fn with_size(size: (f64, f64)) -> Self {
        Self {
            size,
            ..Self::default()
        }
    }

    fn rect(&self) -> Rect<f64> {
        Rect::new(
            self.center
                + Coordinate {
                    x: -self.size.0 / 2.,
                    y: -self.size.1 / 2.,
                },
            self.center
                + Coordinate {
                    x: self.size.0 / 2.,
                    y: self.size.1 / 2.,
                },
        )
    }
}

impl Default for MechanicalSolderPoint {
    fn default() -> Self {
        Self {
            center: [0., 0.].into(),
            size: (1.175, 1.45),
            drill_radius: 0.15,
        }
    }
}

impl fmt::Display for MechanicalSolderPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "msp({:?})", self.size)
    }
}

impl super::InnerFeature for MechanicalSolderPoint {
    fn name(&self) -> &'static str {
        "msp"
    }

    fn translate(&mut self, v: Coordinate<f64>) {
        self.center = self.center + v;
    }

    fn atoms(&self) -> Vec<InnerAtom> {
        vec![
            InnerAtom::Rect {
                layer: Layer::BackCopper,
                rect: self.rect(),
            },
            InnerAtom::Rect {
                layer: Layer::BackMask,
                rect: self.rect(),
            },
            InnerAtom::Rect {
                layer: Layer::FrontCopper,
                rect: self.rect(),
            },
            InnerAtom::Rect {
                layer: Layer::FrontMask,
                rect: self.rect(),
            },
            InnerAtom::Drill {
                center: self.center,
                radius: self.drill_radius,
                plated: true,
            },
        ]
    }
}
