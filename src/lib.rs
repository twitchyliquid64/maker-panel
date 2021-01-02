extern crate geo_booleanop;
use geo::MultiPolygon;
use geo_booleanop::boolean::BooleanOp;
use usvg::NodeExt;

pub mod features;
use features::Feature;

/// Failure modes when constructing or serializing geometry.
#[derive(Debug)]
pub enum Err {
    BadEdgeGeometry(String),
}

/// Combines features into single geometry.
pub struct PanelBuilder<'a> {
    pub features: Vec<Box<dyn Feature + 'a>>,
}

impl<'a> PanelBuilder<'a> {
    /// Constructs a ['PanelBuilder`].
    pub fn new() -> Self {
        let features = Vec::new();
        Self { features }
    }

    /// Constructs a [`PanelBuilder`], pre-sized to hold the given
    /// number of [`features::Feature`] objects before need an allocation.
    pub fn with_capacity(sz: usize) -> Self {
        let features = Vec::with_capacity(sz);
        Self { features }
    }

    /// Adds a feature to the panel.
    pub fn push<F: Feature + 'a>(&mut self, f: F) {
        self.features.push(Box::new(f));
    }

    /// Computes the outer geometry of the panel.
    pub fn edge_geometry(&self) -> Option<MultiPolygon<f64>> {
        self.features
            .iter()
            .map(|f| f.edge())
            .fold(None, |mut acc, g| {
                if let Some(poly) = g {
                    if let Some(current) = acc {
                        acc = Some(poly.union(&current));
                    } else {
                        acc = Some(poly);
                    }
                };
                acc
            })
    }
}

impl std::fmt::Display for PanelBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "builder(")?;
        for feature in &self.features {
            feature.fmt(f)?;
            write!(f, " ")?;
        }
        write!(f, ")")
    }
}

/// Convienence function to produce an SVG tree from the given polygons.
pub fn make_svg(edge: MultiPolygon<f64>) -> Result<usvg::Tree, Err> {
    if edge.iter().count() > 1 {
        return Err(Err::BadEdgeGeometry(
            "multiple polygons provided for edge geometry".to_string(),
        ));
    }

    use geo::bounding_rect::BoundingRect;
    let bounds = edge.bounding_rect().unwrap();

    let size = usvg::Size::new(bounds.width() + 1., bounds.height() + 1.).unwrap();
    let rtree = usvg::Tree::create(usvg::Svg {
        size,
        view_box: usvg::ViewBox {
            rect: size.to_rect(0.0, 0.0),
            aspect: usvg::AspectRatio::default(),
        },
    });

    let mut path = usvg::PathData::new();
    for poly in edge.iter() {
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

    Ok(rtree)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlapping_rects() {
        let mut panel = PanelBuilder::new();
        panel.push(features::Rect::new_with_center([-2.5, -2.5].into(), 5., 5.));
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
