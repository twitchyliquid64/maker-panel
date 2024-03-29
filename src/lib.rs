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
#[cfg(feature = "tessellate")]
mod tessellate;
#[cfg(feature = "tessellate")]
pub use tessellate::normals_from_tessellation;
#[cfg(feature = "tessellate")]
pub use tessellate::{Point as TPoint, TessellationError, VertexBuffers};
#[cfg(feature = "text")]
mod text;

pub use parser::Err as SpecErr;

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
    FabricationInstructions,
}

impl Layer {
    fn color(&self) -> usvg::Color {
        match self {
            Layer::FrontCopper => usvg::Color::new(0x84, 0, 0),
            Layer::FrontMask => usvg::Color::new(0x84, 0, 0x84),
            Layer::FrontLegend => usvg::Color::new(0, 0xce, 0xde),
            Layer::BackCopper => usvg::Color::new(0, 0x84, 0),
            Layer::BackMask => usvg::Color::new(0x84, 0, 0x84),
            Layer::BackLegend => usvg::Color::new(0x4, 0, 0x84),
            Layer::FabricationInstructions => usvg::Color::new(0x66, 0x66, 0x66),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Layer::FrontCopper => String::from("FrontCopper"),
            Layer::FrontMask => String::from("FrontMask"),
            Layer::FrontLegend => String::from("FrontLegend"),
            Layer::BackCopper => String::from("BackCopper"),
            Layer::BackMask => String::from("BackMask"),
            Layer::BackLegend => String::from("BackLegend"),
            Layer::FabricationInstructions => String::from("FabricationInstructions"),
        }
    }
}

/// The direction in which repetitions occur.
#[derive(Debug, Clone, Copy, PartialEq)]
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
    NoBounds,
    BadEdgeGeometry(String),
    InternalGerberFailure,
    #[cfg(feature = "tessellate")]
    TessellationError(TessellationError),
}

/// Combines features into single geometry.
pub struct Panel<'a> {
    pub features: Vec<Box<dyn Feature + 'a>>,
    convex_hull: bool,
    grid_separation: Option<isize>,
}

impl<'a> Panel<'a> {
    /// Constructs a [`Panel`].
    pub fn new() -> Self {
        let features = Vec::new();
        let convex_hull = false;
        let grid_separation = None;
        Self {
            features,
            convex_hull,
            grid_separation,
        }
    }

    /// Constructs a [`Panel`], pre-sized to hold the given
    /// number of [`features::Feature`] objects before need an allocation.
    pub fn with_capacity(sz: usize) -> Self {
        let features = Vec::with_capacity(sz);
        let convex_hull = false;
        let grid_separation = None;
        Self {
            features,
            convex_hull,
            grid_separation,
        }
    }

    /// Enables or disables a convex hull transform on the computed edge geometry.
    pub fn convex_hull(&mut self, convex_hull: bool) {
        self.convex_hull = convex_hull;
    }

    /// Sets the grid separation that should be rendered on the SVG.
    pub fn set_grid_separation(&mut self, grid_separation: Option<isize>) {
        self.grid_separation = grid_separation;
    }

    /// Adds a feature to the panel.
    pub fn push<F: Feature + 'a>(&mut self, f: F) {
        self.features.push(Box::new(f));
    }

    /// Adds the feature described by the given spec to the panel.
    pub fn push_spec(&mut self, spec_str: &str) -> Result<(), SpecErr> {
        self.features.append(&mut parser::build(spec_str)?);
        Ok(())
    }

    /// Returns information about the named geometry in the panel.
    pub fn named_info(&self) -> Vec<features::NamedInfo> {
        self.features.iter().fold(vec![], |mut acc, f| {
            for info in f.named_info() {
                acc.push(info);
            }
            acc
        })
    }

    /// Computes the outer geometry of the panel.
    pub fn edge_geometry(&self) -> Option<MultiPolygon<f64>> {
        let mut edge = self
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

        edge = match (&edge, self.convex_hull) {
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
                    edges
                        .iter()
                        .map(|p| p.interiors())
                        .flatten()
                        .map(|p| p.clone())
                        .collect::<Vec<_>>(),
                );

                Some((vec![poly]).into())
            }
            _ => edge,
        };

        for f in &self.features {
            if let Some(sub) = f.edge_subtract() {
                edge = match edge {
                    Some(e) => Some(e.difference(&sub)),
                    None => None,
                };
            }
        }

        edge
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
        use geo::bounding_rect::BoundingRect;
        let edges = self.edge_poly()?;
        let bounds = edges.bounding_rect().unwrap();

        let commands = gerber::serialize_layer(layer, self.interior_geometry(), bounds)
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

    /// Computes the 2d tessellation of the panel.
    #[cfg(feature = "tessellate")]
    pub fn tessellate_2d(&self) -> Result<VertexBuffers<TPoint, u16>, Err> {
        Ok(
            tessellate::tessellate_2d(self.edge_poly()?, self.interior_geometry())
                .map_err(|e| Err::TessellationError(e))?,
        )
    }

    /// Computes the 3d tessellation of the panel.
    #[cfg(feature = "tessellate")]
    pub fn tessellate_3d(&self) -> Result<(Vec<[f64; 3]>, Vec<u16>), Err> {
        Ok(tessellate::tessellate_3d(self.tessellate_2d()?))
    }

    /// Expands the bounds of the drawing area to give space to any
    /// mechanical / fabrication markings.
    fn expanded_bounds(&self, bounds: geo::Rect<f64>) -> geo::Rect<f64> {
        let ig = self.interior_geometry();
        let has_h_scores = ig.iter().any(|g| matches!(g, InnerAtom::VScoreH(_)));
        let has_v_scores = ig.iter().any(|g| matches!(g, InnerAtom::VScoreV(_)));

        match (has_h_scores, has_v_scores) {
            (true, true) => geo::Rect::<f64>::new(
                bounds.min() - [10., 15.].into(),
                bounds.max() + [65., 65.].into(),
            ),
            (true, false) => geo::Rect::<f64>::new(
                bounds.min() - [10., 5.].into(),
                bounds.max() + [65., 5.].into(),
            ),
            (false, true) => geo::Rect::<f64>::new(
                bounds.min() - [5., 15.].into(),
                bounds.max() + [5., 65.].into(),
            ),
            _ => bounds,
        }
    }

    /// Indicates if the panel has fabrication instructions, such as
    /// V-score lines.
    pub fn has_fab_markings(&self) -> bool {
        let ig = self.interior_geometry();
        let has_h_scores = ig.iter().any(|g| matches!(g, InnerAtom::VScoreH(_)));
        let has_v_scores = ig.iter().any(|g| matches!(g, InnerAtom::VScoreV(_)));

        has_h_scores || has_v_scores
    }

    /// Produces an SVG tree rendering the panel.
    pub fn make_svg(&self) -> Result<usvg::Tree, Err> {
        let edges = self.edge_poly()?;
        use geo::bounding_rect::BoundingRect;
        let bounds = edges.bounding_rect().unwrap();
        let img_bounds = self.expanded_bounds(bounds);

        let size = match usvg::Size::new(img_bounds.width(), img_bounds.height()) {
            Some(sz) => sz,
            None => {
                return Err(Err::NoBounds);
            }
        };
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

        for inners in edges.interiors() {
            let mut path = usvg::PathData::new();
            let mut has_moved = false;
            for point in inners.points_iter() {
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
        }

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
                InnerAtom::Rect {
                    rect: rect_pos,
                    layer: _,
                } => {
                    let p = rect(rect_pos);
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

                InnerAtom::VScoreH(y) => {
                    let mut p = usvg::PathData::with_capacity(2);
                    p.push_move_to(bounds.min().x - 4., y);
                    p.push_line_to(bounds.max().x + 4., y);
                    rtree.root().append_kind(usvg::NodeKind::Path(usvg::Path {
                        stroke: inner.stroke(),
                        fill: inner.fill(),
                        data: std::rc::Rc::new(p),
                        ..usvg::Path::default()
                    }));

                    #[cfg(feature = "text")]
                    rtree
                        .root()
                        .append_kind(usvg::NodeKind::Image(text::blit_text_span(
                            bounds.max().x,
                            y,
                            "v-score".into(),
                        )));
                }
                InnerAtom::VScoreV(x) => {
                    let mut p = usvg::PathData::with_capacity(2);
                    p.push_move_to(x, bounds.min().y - 4.);
                    p.push_line_to(x, bounds.max().y + 4.);
                    rtree.root().append_kind(usvg::NodeKind::Path(usvg::Path {
                        stroke: inner.stroke(),
                        fill: inner.fill(),
                        data: std::rc::Rc::new(p),
                        ..usvg::Path::default()
                    }));
                }
            }
        }

        // for the grid
        if let Some(sep) = self.grid_separation {
            let lower = ((bounds.min().x.floor() as isize) / sep) * sep;
            let upper = ((bounds.max().x.ceil() as isize) / sep) * sep;
            let mut curs: isize = lower;
            while curs <= upper {
                let mut p = usvg::PathData::with_capacity(2);
                p.push_move_to(curs as f64, bounds.min().y);
                p.push_line_to(curs as f64, bounds.max().y);
                rtree.root().append_kind(usvg::NodeKind::Path(usvg::Path {
                    stroke: Some(usvg::Stroke {
                        paint: usvg::Paint::Color(usvg::Color::new(0, 0, 0)),
                        width: usvg::StrokeWidth::new(0.1),
                        dasharray: Some(vec![0.25, 0.75]),
                        linejoin: usvg::LineJoin::Round,
                        ..usvg::Stroke::default()
                    }),
                    data: std::rc::Rc::new(p),
                    ..usvg::Path::default()
                }));

                #[cfg(feature = "text")]
                rtree
                    .root()
                    .append_kind(usvg::NodeKind::Image(text::blit_text_span(
                        curs as f64 + 0.8,
                        bounds.min().y + 0.5,
                        &curs.to_string(),
                    )));

                curs += sep;
            }

            let lower = ((bounds.min().y.floor() as isize) / sep) * sep;
            let upper = ((bounds.max().y.ceil() as isize) / sep) * sep;
            let mut curs: isize = lower;
            while curs <= upper {
                let mut p = usvg::PathData::with_capacity(2);
                p.push_move_to(bounds.min().x, curs as f64);
                p.push_line_to(bounds.max().x, curs as f64);
                rtree.root().append_kind(usvg::NodeKind::Path(usvg::Path {
                    stroke: Some(usvg::Stroke {
                        paint: usvg::Paint::Color(usvg::Color::new(0, 0, 0)),
                        width: usvg::StrokeWidth::new(0.1),
                        dasharray: Some(vec![0.25, 0.75]),
                        linejoin: usvg::LineJoin::Round,
                        ..usvg::Stroke::default()
                    }),
                    data: std::rc::Rc::new(p),
                    ..usvg::Path::default()
                }));

                #[cfg(feature = "text")]
                rtree
                    .root()
                    .append_kind(usvg::NodeKind::Image(text::blit_text_span(
                        bounds.min().x + 0.5,
                        curs as f64 + 0.8,
                        &curs.to_string(),
                    )));

                curs += sep;
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

fn rect(rect: geo::Rect<f64>) -> usvg::PathData {
    let mut p = usvg::PathData::with_capacity(5);
    p.push_move_to(rect.min().x, rect.min().y);
    p.push_line_to(rect.max().x, rect.min().y);
    p.push_line_to(rect.max().x, rect.max().y);
    p.push_line_to(rect.min().x, rect.max().y);
    p.push_line_to(rect.min().x, rect.min().y);
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
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().x > 2.49);
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().x < 2.51);
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().y < -2.49);
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().y > -2.51);
        }
    }

    #[test]
    fn test_array_inner() {
        let mut panel = Panel::new();
        panel.push_spec("[5]R<5>(h3)").unwrap();
        assert_eq!(panel.interior_geometry().len(), 25);

        use geo::bounding_rect::BoundingRect;
        let bounds = panel.edge_geometry().unwrap().bounding_rect().unwrap();
        assert!(bounds.width() > 24.99 && bounds.width() < 25.01);
        assert!(bounds.height() > 4.99 && bounds.height() < 5.01);
    }

    #[test]
    fn test_column_down() {
        let mut panel = Panel::new();
        panel
            .push_spec("column left { R<5,5>(h) R<3>(h) } ")
            .unwrap();

        use geo::bounding_rect::BoundingRect;
        let bounds = panel.edge_geometry().unwrap().bounding_rect().unwrap();
        eprintln!("{:?}\n\n{:?}", panel.features, bounds);
        assert!(bounds.width() > 4.99 && bounds.width() < 5.01);
        assert!(bounds.height() > 7.99 && bounds.height() < 8.01);
    }

    #[test]
    fn test_circ_inner() {
        let mut panel = Panel::new();
        panel.push_spec("C<5>(h2)").unwrap();

        for i in 0..5 {
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().x > -0.01);
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().x < 0.01);
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().y < 0.01);
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().y > -0.01);
        }

        let mut panel = Panel::new();
        panel.push_spec("C<@(1, 1), 1>(h2)").unwrap();
        // eprintln!("{:?}", panel.interior_geometry());
        for i in 0..5 {
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().x > 0.99);
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().x < 1.01);
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().y < 1.01);
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().y > 0.99);
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
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().x < 3.01);
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().x > 2.99);
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().y < 2.01);
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().y > -2.01);
        }
        for i in 5..10 {
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().x < 5.01);
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().x > 4.99);
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().y < 2.01);
            assert!(panel.interior_geometry()[i].bounds().unwrap().center().y > -2.01);
        }
    }

    #[test]
    fn test_atpos_angle() {
        let mut r = features::AtPos::<features::Rect, features::Rect>::new(
            features::Rect::with_center([4., 2.].into(), 1., 1.),
        );
        r.push(
            features::Rect::with_center([0., 0.].into(), 1., 1.),
            features::Positioning::Angle {
                degrees: 45.,
                amount: 3.,
            },
        );
        let ig = r.edge_union().unwrap();
        use geo::prelude::Contains;
        assert!(ig.contains(&geo::Coordinate::from([5.8, 3.8])));
    }

    #[test]
    fn test_atpos_corner() {
        let mut r = features::AtPos::<features::Rect, features::Rect>::new(
            features::Rect::with_center([2., 2.].into(), 4., 4.),
        );
        r.push(
            features::Rect::with_center([0., 0.].into(), 0.6, 0.6),
            features::Positioning::Corner {
                side: Direction::Left,
                align: Align::End,
                opposite: false,
            },
        );
        let ig = r.edge_union().unwrap();
        use geo::prelude::Contains;
        assert!(ig.contains(&geo::Coordinate::from([-0.5, 0.5])));

        let mut r = features::AtPos::<features::Rect, features::Rect>::new(
            features::Rect::with_center([2., 2.].into(), 4., 4.),
        );
        r.push(
            features::Rect::with_center([0., 0.].into(), 0.6, 0.6),
            features::Positioning::Corner {
                side: Direction::Up,
                align: Align::End,
                opposite: true,
            },
        );
        let ig = r.edge_union().unwrap();
        // eprintln!("{:?}", ig);
        assert!(ig.contains(&geo::Coordinate::from([3.9, -0.5])));
    }

    #[test]
    fn test_cel_basic() {
        let mut panel = Panel::new();
        panel.push_spec("let ye = !{2};\nR<$ye>").unwrap();

        use geo::bounding_rect::BoundingRect;
        let bounds = panel.edge_geometry().unwrap().bounding_rect().unwrap();
        // eprintln!("{:?}\n\n{:?}", panel.features, bounds);
        assert!(bounds.width() > 1.99 && bounds.width() < 2.01);
        assert!(bounds.height() > 1.99 && bounds.height() < 2.01);

        let mut panel = Panel::new();
        panel.push_spec("let ye = !{2.0};\nR<$ye>").unwrap();

        let bounds = panel.edge_geometry().unwrap().bounding_rect().unwrap();
        // eprintln!("{:?}\n\n{:?}", panel.features, bounds);
        assert!(bounds.width() > 1.99 && bounds.width() < 2.01);
        assert!(bounds.height() > 1.99 && bounds.height() < 2.01);
    }

    #[test]
    fn test_cel_expr() {
        let mut panel = Panel::new();
        panel
            .push_spec("let ye = !{2 + 2.0};\nR<!{2}, $ye>")
            .unwrap();

        use geo::bounding_rect::BoundingRect;
        let bounds = panel.edge_geometry().unwrap().bounding_rect().unwrap();
        // eprintln!("{:?}\n\n{:?}", panel.features, bounds);
        assert!(bounds.width() > 1.99 && bounds.width() < 2.01);
        assert!(bounds.height() > 3.99 && bounds.height() < 4.01);
    }

    #[test]
    fn test_cel_wrap() {
        let mut panel = Panel::new();
        panel
            .push_spec("let ye = !{2 + 1.0};\nwrap(R<!{2}>) with { left align exterior => R<!{ye + 1}>,\n }")
            .unwrap();

        use geo::bounding_rect::BoundingRect;
        let bounds = panel.edge_geometry().unwrap().bounding_rect().unwrap();
        // eprintln!("{:?}\n\n{:?}", panel.features, bounds);
        assert!(bounds.width() > 5.99 && bounds.width() < 6.01);
        assert!(bounds.height() > 3.99 && bounds.height() < 4.01);
    }

    #[test]
    fn test_rotate() {
        let mut panel = Panel::new();
        panel.push_spec("rotate(90) { C<2.5> }").unwrap();

        use geo::bounding_rect::BoundingRect;
        let bounds = panel.edge_geometry().unwrap().bounding_rect().unwrap();
        eprintln!("{:?}\n\n{:?}", panel.features, bounds);
        assert!(bounds.width() > 4.99 && bounds.width() < 5.0001);
        assert!(bounds.height() > 4.99 && bounds.height() < 5.0001);
    }

    #[test]
    fn test_named() {
        let mut panel = Panel::new();
        panel
            .push_spec("wrap([2]R<3> % inner) with { left align exterior => R<2> % rect }")
            .unwrap();
        //eprintln!("{:?}\n", panel.features);

        let infos = panel.named_info();
        //eprintln!("{:?}\n\n", infos);
        assert!(infos.len() == 3 && infos[0].name == "inner0" && infos[0].bounds.min().x < -1.499);
        assert!(infos.len() == 3 && infos[1].name == "inner1" && infos[1].bounds.min().x < 1.5001);
        assert!(infos.len() == 3 && infos[2].name == "rect" && infos[2].bounds.min().x < -3.4999);
    }
}
