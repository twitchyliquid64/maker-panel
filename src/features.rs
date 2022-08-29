//! Components which compose a panel.

use dyn_clone::DynClone;
use geo::{Coordinate, MultiPolygon};
use std::fmt;

mod array;
mod circle;
mod mechanical_solder_point;
mod named;
mod negative;
mod pos;
mod r_mount;
mod rect;
pub mod repeating;
mod rotate;
mod screw_hole;
mod smiley;
mod triangle;
mod unit;
pub use array::Column;
pub use circle::Circle;
pub use mechanical_solder_point::MechanicalSolderPoint;
pub use named::Named;
pub use negative::Negative;
pub use pos::{AtPos, Positioning};
pub use r_mount::RMount;
pub use rect::Rect;
pub use rotate::Rotate;
pub use screw_hole::ScrewHole;
pub use smiley::Smiley;
pub use triangle::Triangle;
pub use unit::Unit;

/// Describes named geometry.
#[derive(Debug, Clone)]
pub struct NamedInfo {
    pub name: String,
    pub bounds: geo::Rect<f64>,
}

impl NamedInfo {
    pub fn new(name: String, bounds: geo::Rect<f64>) -> Self {
        NamedInfo { name, bounds }
    }
    pub fn translate(&mut self, x: f64, y: f64) {
        use geo::prelude::Translate;
        self.bounds.translate_inplace(x, y);
    }
    pub fn name_index(&mut self, idx: usize) {
        self.name = self.name.clone() + &idx.to_string();
    }
}

/// Specifies geometry interior to the bounds of the panel.
pub trait InnerFeature: fmt::Display + DynClone + fmt::Debug {
    fn name(&self) -> &'static str;
    fn translate(&mut self, v: Coordinate<f64>);
    fn atoms(&self) -> Vec<InnerAtom>;
}

dyn_clone::clone_trait_object!(InnerFeature);

impl<'a> InnerFeature for Box<dyn InnerFeature + 'a> {
    fn name(&self) -> &'static str {
        self.as_ref().name()
    }

    fn translate(&mut self, v: Coordinate<f64>) {
        self.as_mut().translate(v)
    }

    fn atoms(&self) -> Vec<InnerAtom> {
        self.as_ref().atoms()
    }
}

/// A top-level unit that makes up the geometry of the panel.
pub trait Feature: fmt::Display + DynClone + fmt::Debug {
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

    /// named_info returns information about named geometry.
    fn named_info(&self) -> Vec<NamedInfo> {
        vec![]
    }
}

dyn_clone::clone_trait_object!(Feature);

impl<'a> Feature for Box<dyn Feature + 'a> {
    fn name(&self) -> &'static str {
        self.as_ref().name()
    }

    fn translate(&mut self, v: Coordinate<f64>) {
        self.as_mut().translate(v)
    }

    fn edge_union(&self) -> Option<MultiPolygon<f64>> {
        self.as_ref().edge_union()
    }

    fn interior(&self) -> Vec<InnerAtom> {
        self.as_ref().interior()
    }

    fn edge_subtract(&self) -> Option<MultiPolygon<f64>> {
        self.as_ref().edge_subtract()
    }

    fn named_info(&self) -> Vec<NamedInfo> {
        self.as_ref().named_info()
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
    Rect {
        rect: geo::Rect<f64>,
        layer: super::Layer,
    },
    VScoreH(f64),
    VScoreV(f64),
}

impl InnerAtom {
    pub fn stroke(&self) -> Option<usvg::Stroke> {
        match self {
            InnerAtom::VScoreH(_) | InnerAtom::VScoreV(_) => Some(usvg::Stroke {
                paint: usvg::Paint::Color(usvg::Color::new(0x6, 0x6, 0x6)),
                width: usvg::StrokeWidth::new(0.1),
                opacity: usvg::Opacity::new(0.5),
                dasharray: Some(vec![0.8, 0.8]),
                ..usvg::Stroke::default()
            }),
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
            InnerAtom::Rect { layer, .. } => Some(usvg::Fill {
                paint: usvg::Paint::Color(layer.color()),
                ..usvg::Fill::default()
            }),
            InnerAtom::VScoreH(_) | InnerAtom::VScoreV(_) => None,
        }
    }

    pub fn bounds(&self) -> Option<geo::Rect<f64>> {
        match self {
            InnerAtom::Drill { center, radius, .. } => Some(geo::Rect::new(
                Coordinate {
                    x: center.x - radius,
                    y: center.y - radius,
                },
                Coordinate {
                    x: center.x + radius,
                    y: center.y + radius,
                },
            )),
            InnerAtom::Circle { center, radius, .. } => Some(geo::Rect::new(
                Coordinate {
                    x: center.x - radius,
                    y: center.y - radius,
                },
                Coordinate {
                    x: center.x + radius,
                    y: center.y + radius,
                },
            )),
            InnerAtom::Rect { rect, .. } => Some(rect.clone()),
            InnerAtom::VScoreH(_) | InnerAtom::VScoreV(_) => None,
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
            InnerAtom::Rect { rect, .. } => {
                use geo::algorithm::translate::Translate;
                rect.translate_inplace(x, y);
            }
            InnerAtom::VScoreH(ref mut y2) => {
                *y2 = *y2 + y;
            }
            InnerAtom::VScoreV(ref mut x2) => {
                *x2 = *x2 + x;
            }
        }
    }
}
