use geo::{Coordinate, LineString, MultiPolygon};
use std::fmt;

/// A single unit that makes up the geometry of the panel.
pub trait Feature: fmt::Display {
    fn name(&self) -> &'static str;
    fn id(&self) -> Option<String>;
    fn edge(&self) -> Option<MultiPolygon<f64>>;
}

/// A rectangular region with square edges.
#[derive(Debug, Clone)]
pub struct Rect {
    rect: geo::Rect<f64>,
}

impl Rect {
    /// Constructs a new rectangle using the provided corners.
    pub fn new(top_left: Coordinate<f64>, bottom_right: Coordinate<f64>) -> Self {
        Self {
            rect: geo::Rect::new(top_left, bottom_right),
        }
    }

    /// Constructs a new rectangle given a center point and sizes.
    pub fn new_with_center(center: Coordinate<f64>, width: f64, height: f64) -> Self {
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
        }
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rect({:?}, {:?})", self.rect.min(), self.rect.max())
    }
}

impl Feature for Rect {
    fn name(&self) -> &'static str {
        "rect"
    }

    fn id(&self) -> Option<String> {
        None
    }

    fn edge(&self) -> Option<MultiPolygon<f64>> {
        Some(self.rect.clone().to_polygon().into())
    }
}
