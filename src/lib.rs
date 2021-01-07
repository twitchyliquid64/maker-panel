extern crate conv;
extern crate geo_booleanop;
extern crate gerber_types;

use geo::{Coordinate, MultiPolygon};
use geo_booleanop::boolean::BooleanOp;
use usvg::NodeExt;

pub mod features;
use features::{Feature, InnerAtom};

mod drill;
mod gerber;
mod parser;

/// Alignment of multiple elements in an array.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
}

/// PCB layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Layer {
    FrontCopper,
    FrontMask,
    FrontLegend,
    BackCopper,
    BackMask,
    BackLegend,
}

impl Layer {
    fn color(&self) -> usvg::Color {
        match self {
            Layer::FrontCopper => usvg::Color::new(0x84, 0, 0),
            Layer::FrontMask => usvg::Color::new(0x84, 0, 0x84),
            Layer::FrontLegend => usvg::Color::new(0, 0, 0x84),
            Layer::BackCopper => usvg::Color::new(0, 0x84, 0),
            Layer::BackMask => usvg::Color::new(0x84, 0, 0x84),
            Layer::BackLegend => usvg::Color::new(0x4, 0, 0x84),
        }
    }
}

/// The direction in which repetitions occur.
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Down,
    Up,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Direction::Left => write!(f, "left"),
            Direction::Right => write!(f, "right"),
            Direction::Down => write!(f, "down"),
            Direction::Up => write!(f, "up"),
        }
    }
}

impl Direction {
    pub fn offset(&self, bounds: geo::Rect<f64>) -> (f64, f64) {
        match self {
            Direction::Left => (-bounds.width(), 0.0),
            Direction::Right => (bounds.width(), 0.0),
            Direction::Down => (0.0, bounds.height()),
            Direction::Up => (0.0, -bounds.height()),
        }
    }
}

/// Failure modes when constructing or serializing geometry.
#[derive(Debug)]
pub enum Err {
    NoFeatures,
    BadEdgeGeometry(String),
    InternalGerberFailure,
}

/// Combines features into single geometry.
pub struct Panel<'a> {
    pub features: Vec<Box<dyn Feature + 'a>>,
    convex_hull: bool,
}

impl<'a> Panel<'a> {
    /// Constructs a [`Panel`].
    pub fn new() -> Self {
        let features = Vec::new();
        let convex_hull = false;
        Self {
            features,
            convex_hull,
        }
    }

    /// Constructs a [`Panel`], pre-sized to hold the given
    /// number of [`features::Feature`] objects before need an allocation.
    pub fn with_capacity(sz: usize) -> Self {
        let features = Vec::with_capacity(sz);
        let convex_hull = false;
        Self {
            features,
            convex_hull,
        }
    }

    /// Enables or disables a convex hull transform on the computed edge geometry.
    pub fn convex_hull(&mut self, convex_hull: bool) {
        self.convex_hull = convex_hull;
    }

    /// Adds a feature to the panel.
    pub fn push<F: Feature + 'a>(&mut self, f: F) {
        self.features.push(Box::new(f));
    }

    /// Adds the feature described by the given spec to the panel.
    pub fn push_spec(&mut self, spec_str: &str) -> Result<(), ()> {
        self.features.append(&mut parser::build(spec_str)?);
        Ok(())
    }

    /// Computes the outer geometry of the panel.
    pub fn edge_geometry(&self) -> Option<MultiPolygon<f64>> {
        let edge = self
            .features
            .iter()
            .map(|f| f.edge_union())
            .fold(None, |mut acc, g| {
                if let Some(poly) = g {
                    if let Some(current) = acc {
                        acc = Some(poly.union(&current));
                    } else {
                        acc = Some(poly);
                    }
                };
                acc
            });

        match (&edge, self.convex_hull) {
            (Some(edges), true) => {
                use geo::algorithm::convex_hull;
                let mut points = edges
                    .iter()
                    .map(|p| p.exterior().points_iter().collect::<Vec<_>>())
                    .flatten()
                    .map(|p| p.into())
                    .collect::<Vec<Coordinate<_>>>();

                let poly = geo::Polygon::new(
                    convex_hull::graham::graham_hull(points.as_mut_slice(), true),
                    vec![],
                );

                Some((vec![poly]).into())
            }
            _ => edge,
        }
    }

    fn edge_poly(&self) -> Result<geo::Polygon<f64>, Err> {
        match self.edge_geometry() {
            Some(edges) => {
                let mut polys = edges.into_iter();
                match polys.len() {
                    0 => Err(Err::NoFeatures),
                    1 => Ok(polys.next().unwrap()),
                    _ => Err(Err::BadEdgeGeometry(
                        "multiple polygons provided for edge geometry".to_string(),
                    )),
                }
            }
            None => Err(Err::NoFeatures),
        }
    }

    /// Computes the inner geometry of the panel.
    pub fn interior_geometry(&self) -> Vec<InnerAtom> {
        self.features
            .iter()
            .map(|f| f.interior())
            .flatten()
            .collect()
    }

    /// Serializes a gerber file describing the PCB profile to the provided writer.
    pub fn serialize_gerber_edges<W: std::io::Write>(&self, w: &mut W) -> Result<(), Err> {
        let edges = self.edge_poly()?;
        let commands = gerber::serialize_edge(edges).map_err(|_| Err::InternalGerberFailure)?;
        use gerber_types::GerberCode;
        commands
            .serialize(w)
            .map_err(|_| Err::InternalGerberFailure)
    }

    /// Serializes a gerber file describing the layer (copper or soldermask) to
    /// to the provided writer.
    pub fn serialize_gerber_layer<W: std::io::Write>(
        &self,
        layer: Layer,
        w: &mut W,
    ) -> Result<(), Err> {
        let commands = gerber::serialize_layer(layer, self.interior_geometry())
            .map_err(|_| Err::InternalGerberFailure)?;
        use gerber_types::GerberCode;
        commands
            .serialize(w)
            .map_err(|_| Err::InternalGerberFailure)
    }

    /// Serializes a drill file describing drill hits to the provided writer.
    pub fn serialize_drill<W: std::io::Write>(
        &self,
        w: &mut W,
        want_plated: bool,
    ) -> Result<(), std::io::Error> {
        drill::serialize(&self.interior_geometry(), w, want_plated)
    }

    /// Produces an SVG tree rendering the panel.
    pub fn make_svg(&self) -> Result<usvg::Tree, Err> {
        let edges = self.edge_poly()?;
        use geo::bounding_rect::BoundingRect;
        let bounds = edges.bounding_rect().unwrap();

        let size = usvg::Size::new(bounds.width(), bounds.height()).unwrap();
        let rtree = usvg::Tree::create(usvg::Svg {
            size,
            view_box: usvg::ViewBox {
                rect: size.to_rect(0.0, 0.0),
                aspect: usvg::AspectRatio::default(),
            },
        });

        let mut path = usvg::PathData::new();
        let mut has_moved = false;
        for point in edges.exterior().points_iter() {
            if !has_moved {
                has_moved = true;
                path.push_move_to(point.x(), point.y());
            } else {
                path.push_line_to(point.x(), point.y());
            }
        }
        path.push_close_path();

        rtree.root().append_kind(usvg::NodeKind::Path(usvg::Path {
            stroke: Some(usvg::Stroke {
                paint: usvg::Paint::Color(usvg::Color::new(0, 0, 0)),
                width: usvg::StrokeWidth::new(0.1),
                ..usvg::Stroke::default()
            }),
            data: std::rc::Rc::new(path),
            ..usvg::Path::default()
        }));

        for inner in self.interior_geometry() {
            match inner {
                InnerAtom::Circle { center, radius, .. } => {
                    let p = circle(center, radius);
                    rtree.root().append_kind(usvg::NodeKind::Path(usvg::Path {
                        stroke: inner.stroke(),
                        fill: inner.fill(),
                        data: std::rc::Rc::new(p),
                        ..usvg::Path::default()
                    }));
                }
                InnerAtom::Drill { center, radius, .. } => {
                    let p = circle(center, radius);
                    rtree.root().append_kind(usvg::NodeKind::Path(usvg::Path {
                        stroke: inner.stroke(),
                        fill: inner.fill(),
                        data: std::rc::Rc::new(p),
                        ..usvg::Path::default()
                    }));
                }
            }
        }

        Ok(rtree)
    }
}

fn circle(center: Coordinate<f64>, radius: f64) -> usvg::PathData {
    let mut p = usvg::PathData::with_capacity(6);
    p.push_move_to(center.x + radius, center.y);
    p.push_arc_to(
        radius,
        radius,
        0.0,
        false,
        true,
        center.x,
        center.y + radius,
    );
    p.push_arc_to(
        radius,
        radius,
        0.0,
        false,
        true,
        center.x - radius,
        center.y,
    );
    p.push_arc_to(
        radius,
        radius,
        0.0,
        false,
        true,
        center.x,
        center.y - radius,
    );
    p.push_arc_to(
        radius,
        radius,
        0.0,
        false,
        true,
        center.x + radius,
        center.y,
    );
    p.push_close_path();
    p
}

impl std::fmt::Display for Panel<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "panel(")?;
        for feature in &self.features {
            feature.fmt(f)?;
            write!(f, " ")?;
        }
        write!(f, ")")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlapping_rects() {
        let mut panel = Panel::new();
        panel.push_spec("R<@(-2.5, -2.5), 5>(h3)").unwrap();
        panel.push(features::Rect::new([-0., -1.].into(), [5., 3.].into()));

        assert_eq!(
            panel.edge_geometry().unwrap(),
            geo::MultiPolygon(vec![geo::Polygon::new(
                geo::LineString(vec![
                    geo::Coordinate { x: -5.0, y: -5.0 },
                    geo::Coordinate { x: 0.0, y: -5.0 },
                    geo::Coordinate { x: 0.0, y: -1.0 },
                    geo::Coordinate { x: 5.0, y: -1.0 },
                    geo::Coordinate { x: 5.0, y: 3.0 },
                    geo::Coordinate { x: -0.0, y: 3.0 },
                    geo::Coordinate { x: 0.0, y: 0.0 },
                    geo::Coordinate { x: -5.0, y: 0.0 },
                    geo::Coordinate { x: -5.0, y: -5.0 }
                ]),
                vec![],
            )]),
        );
    }

    #[test]
    fn test_rect_inner() {
        let mut panel = Panel::new();
        panel.push_spec("R<@(2.5, -2.5), 5>(h3)").unwrap();

        // eprintln!("{:?}", panel.interior_geometry());
        for i in 0..5 {
            assert!(panel.interior_geometry()[i].bounds().center().x > 2.49);
            assert!(panel.interior_geometry()[i].bounds().center().x < 2.51);
            assert!(panel.interior_geometry()[i].bounds().center().y < -2.49);
            assert!(panel.interior_geometry()[i].bounds().center().y > -2.51);
        }
    }

    #[test]
    fn test_circ_inner() {
        let mut panel = Panel::new();
        panel.push_spec("C<5>(h2)").unwrap();

        for i in 0..5 {
            assert!(panel.interior_geometry()[i].bounds().center().x > -0.01);
            assert!(panel.interior_geometry()[i].bounds().center().x < 0.01);
            assert!(panel.interior_geometry()[i].bounds().center().y < 0.01);
            assert!(panel.interior_geometry()[i].bounds().center().y > -0.01);
        }

        let mut panel = Panel::new();
        panel.push_spec("C<@(1, 1), 1>(h2)").unwrap();
        // eprintln!("{:?}", panel.interior_geometry());
        for i in 0..5 {
            assert!(panel.interior_geometry()[i].bounds().center().x > 0.99);
            assert!(panel.interior_geometry()[i].bounds().center().x < 1.01);
            assert!(panel.interior_geometry()[i].bounds().center().y < 1.01);
            assert!(panel.interior_geometry()[i].bounds().center().y > 0.99);
        }
    }

    #[test]
    fn test_atpos_xends() {
        let mut panel = Panel::new();
        panel.push(features::AtPos::x_ends(
            features::Rect::with_center([4., 2.].into(), 2., 3.),
            Some(features::Circle::wrap_with_radius(
                features::ScrewHole::with_diameter(1.),
                2.,
            )),
            Some(features::Circle::wrap_with_radius(
                features::ScrewHole::with_diameter(1.),
                2.,
            )),
        ));

        for i in 0..5 {
            assert!(panel.interior_geometry()[i].bounds().center().x < 3.01);
            assert!(panel.interior_geometry()[i].bounds().center().x > 2.99);
            assert!(panel.interior_geometry()[i].bounds().center().y < 2.01);
            assert!(panel.interior_geometry()[i].bounds().center().y > -2.01);
        }
        for i in 5..10 {
            assert!(panel.interior_geometry()[i].bounds().center().x < 5.01);
            assert!(panel.interior_geometry()[i].bounds().center().x > 4.99);
            assert!(panel.interior_geometry()[i].bounds().center().y < 2.01);
            assert!(panel.interior_geometry()[i].bounds().center().y > -2.01);
        }
    }
}
