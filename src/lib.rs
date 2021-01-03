extern crate geo_booleanop;
use geo::{Coordinate, MultiPolygon};
use geo_booleanop::boolean::BooleanOp;
use usvg::NodeExt;

pub mod features;
use features::{Feature, InnerAtom};

/// PCB layers.
#[derive(Debug, Clone)]
pub enum Layer {
    FrontCopper,
    FrontMask,
    BackCopper,
    BackMask,
}

impl Layer {
    fn color(&self) -> usvg::Color {
        match self {
            Layer::FrontCopper => usvg::Color::new(0x84, 0, 0),
            Layer::FrontMask => usvg::Color::new(0x84, 0, 0x84),
            Layer::BackCopper => usvg::Color::new(0, 0x84, 0),
            Layer::BackMask => usvg::Color::new(0x84, 0, 0x84),
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

    /// Computes the inner geometry of the panel.
    pub fn interior_geometry(&self) -> Vec<InnerAtom> {
        self.features
            .iter()
            .map(|f| f.interior())
            .flatten()
            .collect()
    }

    /// Produces an SVG tree rendering the panel.
    pub fn make_svg(&self) -> Result<usvg::Tree, Err> {
        let edges = match self.edge_geometry() {
            Some(edges) => edges,
            None => {
                return Err(Err::NoFeatures);
            }
        };

        if edges.iter().count() > 1 {
            // println!("failing geo = {:?}\n\n", edges);
            return Err(Err::BadEdgeGeometry(
                "multiple polygons provided for edge geometry".to_string(),
            ));
        }

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
        for poly in edges.iter() {
            let mut has_moved = false;
            for point in poly.exterior().points_iter() {
                if !has_moved {
                    has_moved = true;
                    path.push_move_to(point.x(), point.y());
                } else {
                    path.push_line_to(point.x(), point.y());
                }
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
        panel.push(features::Rect::with_center([-2.5, -2.5].into(), 5., 5.));
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
}
