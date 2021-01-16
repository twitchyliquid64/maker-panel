use super::InnerAtom;
use crate::Layer;
use geo::{Coordinate, Rect};
use std::fmt;

/// An interior feature that just renders a smiley on the front silkscreen.
#[derive(Debug, Clone)]
pub struct Smiley {
    center: Coordinate<f64>,
}

impl Default for Smiley {
    fn default() -> Self {
        Self {
            center: [0., 0.].into(),
        }
    }
}

impl fmt::Display for Smiley {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "smiley(center = {:?})", self.center,)
    }
}

impl super::InnerFeature for Smiley {
    fn name(&self) -> &'static str {
        "smiley"
    }

    fn translate(&mut self, v: Coordinate<f64>) {
        self.center = self.center + v;
    }

    fn atoms(&self) -> Vec<InnerAtom> {
        vec![
            InnerAtom::Circle {
                center: self.center
                    + Coordinate {
                        x: -0.6f64,
                        y: -0.6f64,
                    },
                radius: 0.4,
                layer: Layer::FrontLegend,
            },
            InnerAtom::Circle {
                center: self.center
                    + Coordinate {
                        x: 0.6f64,
                        y: -0.6f64,
                    },
                radius: 0.4,
                layer: Layer::FrontLegend,
            },
            InnerAtom::Rect {
                layer: Layer::FrontLegend,
                rect: Rect::new(
                    self.center
                        + Coordinate {
                            x: -1.4f64,
                            y: 0.15f64,
                        },
                    self.center
                        + Coordinate {
                            x: -1.0f64,
                            y: 0.9f64,
                        },
                ),
            },
            InnerAtom::Rect {
                layer: Layer::FrontLegend,
                rect: Rect::new(
                    self.center
                        + Coordinate {
                            x: -1.0f64,
                            y: 0.6f64,
                        },
                    self.center
                        + Coordinate {
                            x: 1.0f64,
                            y: 0.9f64,
                        },
                ),
            },
            InnerAtom::Rect {
                layer: Layer::FrontLegend,
                rect: Rect::new(
                    self.center
                        + Coordinate {
                            x: 1.0f64,
                            y: 0.9f64,
                        },
                    self.center
                        + Coordinate {
                            x: 1.4f64,
                            y: 0.15f64,
                        },
                ),
            },
        ]
    }
}
