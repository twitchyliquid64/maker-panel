//! Components which compose a panel.

use geo::{Coordinate, MultiPolygon};
use std::fmt;

mod array;
mod circle;
mod pos;
mod rect;
pub mod repeating;
mod screw_hole;
mod unit;
pub use array::Column;
pub use circle::Circle;
pub use pos::AtPos;
pub use rect::Rect;
pub use screw_hole::ScrewHole;
pub use unit::Unit;

/// Specifies geometry interior to the bounds of the panel.
pub trait InnerFeature: fmt::Display {
    fn name(&self) -> &'static str;
    fn translate(&mut self, v: Coordinate<f64>);
    fn atoms(&self) -> Vec<InnerAtom>;
}

/// A top-level unit that makes up the geometry of the panel.
pub trait Feature: fmt::Display {
    /// Human-readable name describing the construction.
    fn name(&self) -> &'static str;
    /// Adjust all coordinates by the specified amount. Should
    /// affect all geometries returned from [`Feature::edge_union`],
    /// [`Feature::edge_subtract`], and [`Feature::interior`].
    fn translate(&mut self, v: Coordinate<f64>);

    /// Returns the outer geometry describing the boundaries of the
    /// panel, which should be unioned with the outer geometry of all
    /// other features.
    fn edge_union(&self) -> Option<MultiPolygon<f64>>;

    /// Returns the inner geometry describing features on the panel,
    /// within the bounds of the computed edge geometry.
    fn interior(&self) -> Vec<InnerAtom>;

    /// Returns the outer geometry describing the boundaries of the
    /// panel, which should be subtracted from the outer geometry of all
    /// other features.
    fn edge_subtract(&self) -> Option<MultiPolygon<f64>> {
        None
    }
}

/// The smallest geometries from which inner features are composed.
#[derive(Debug, Clone)]
pub enum InnerAtom {
    Drill {
        center: Coordinate<f64>,
        radius: f64,
        plated: bool,
    },
    Circle {
        center: Coordinate<f64>,
        radius: f64,
        layer: super::Layer,
    },
}

impl InnerAtom {
    pub fn stroke(&self) -> Option<usvg::Stroke> {
        match self {
            // InnerAtom::Circle { layer, .. } => Some(usvg::Stroke {
            //     paint: usvg::Paint::Color(layer.color()),
            //     width: usvg::StrokeWidth::new(0.1),
            //     opacity: usvg::Opacity::new(0.5),
            //     ..usvg::Stroke::default()
            // }),
            _ => None,
        }
    }

    pub fn fill(&self) -> Option<usvg::Fill> {
        match self {
            InnerAtom::Drill { .. } => Some(usvg::Fill {
                paint: usvg::Paint::Color(usvg::Color::new(0x25, 0x25, 0x25)),
                ..usvg::Fill::default()
            }),
            InnerAtom::Circle { layer, .. } => Some(usvg::Fill {
                paint: usvg::Paint::Color(layer.color()),
                ..usvg::Fill::default()
            }),
        }
    }

    pub fn bounds(&self) -> geo::Rect<f64> {
        match self {
            InnerAtom::Drill { center, radius, .. } => geo::Rect::new(
                Coordinate {
                    x: center.x - radius,
                    y: center.y - radius,
                },
                Coordinate {
                    x: center.x + radius,
                    y: center.y + radius,
                },
            ),
            InnerAtom::Circle { center, radius, .. } => geo::Rect::new(
                Coordinate {
                    x: center.x - radius,
                    y: center.y - radius,
                },
                Coordinate {
                    x: center.x + radius,
                    y: center.y + radius,
                },
            ),
        }
    }

    pub fn translate(&mut self, x: f64, y: f64) {
        match self {
            InnerAtom::Drill { ref mut center, .. } => {
                *center = *center + Coordinate { x, y };
            }
            InnerAtom::Circle { center, .. } => {
                *center = *center + Coordinate { x, y };
            }
        }
    }
}
