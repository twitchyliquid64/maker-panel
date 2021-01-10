use crate::Direction;
use geo::{Coordinate, MultiPolygon};
use std::fmt;

/// How a feature should be positioned relative to an inner feature.
#[derive(Debug, Clone)]
pub struct Positioning {
    pub side: Direction,
    pub centerline_adjustment: f64,
}

impl Positioning {
    fn compute_translation(&self, bounds: geo::Rect<f64>, feature: geo::Rect<f64>) -> (f64, f64) {
        match self.side {
            Direction::Left => (
                bounds.min().x - feature.center().x,
                bounds.center().y - feature.center().y
                    + (self.centerline_adjustment * bounds.height()),
            ),
            Direction::Right => (
                bounds.max().x - feature.center().x,
                bounds.center().y - feature.center().y
                    + (self.centerline_adjustment * bounds.height()),
            ),
            Direction::Up => (
                bounds.center().x - feature.center().x
                    + (self.centerline_adjustment * bounds.width()),
                bounds.min().y - feature.center().y,
            ),
            Direction::Down => (
                bounds.center().x - feature.center().x
                    + (self.centerline_adjustment * bounds.width()),
                bounds.max().y - feature.center().y,
            ),
        }
    }
}

/// A wrapper around a feature that can position other features.
#[derive(Debug, Clone)]
pub struct AtPos<U = super::Unit, S = super::Unit> {
    inner: U,
    elements: Vec<(S, Positioning)>,
}

impl<U, S> AtPos<U, S>
where
    U: super::Feature + std::fmt::Debug + Clone,
    S: super::Feature + std::fmt::Debug + Clone,
{
    /// Constructs a feature that positions the centeroid of other
    /// features at the left & right points of the primary feature.
    pub fn x_ends(primary: U, left: Option<S>, right: Option<S>) -> Self {
        let mut elements = Vec::with_capacity(2);
        if let Some(left) = left {
            elements.push((
                left,
                Positioning {
                    side: Direction::Left,
                    centerline_adjustment: 0.0,
                },
            ));
        }
        if let Some(right) = right {
            elements.push((
                right,
                Positioning {
                    side: Direction::Right,
                    centerline_adjustment: 0.0,
                },
            ));
        }
        Self {
            elements,
            inner: primary,
        }
    }

    /// Wraps a feature so others can be positioned around it.
    pub fn new(primary: U) -> Self {
        let elements = Vec::with_capacity(4);
        Self {
            elements,
            inner: primary,
        }
    }

    /// Constructs a feature that positions the centeroid of the other
    /// feature to the left of the primary feature.
    pub fn left(primary: U, left: S) -> Self {
        let mut elements = Vec::with_capacity(2);
        elements.push((
            left,
            Positioning {
                side: Direction::Left,
                centerline_adjustment: 0.0,
            },
        ));

        Self {
            elements,
            inner: primary,
        }
    }

    /// Constructs a feature that positions the centeroid of the other
    /// feature to the right of the primary feature.
    pub fn right(primary: U, right: S) -> Self {
        let mut elements = Vec::with_capacity(2);
        elements.push((
            right,
            Positioning {
                side: Direction::Right,
                centerline_adjustment: 0.0,
            },
        ));

        Self {
            elements,
            inner: primary,
        }
    }

    /// Adds a feature to be positioned relative to the inner feature,
    /// according to the provided positioning parameters.
    pub fn push(&mut self, feature: S, pos: Positioning) {
        self.elements.push((feature, pos));
    }
}

fn compute_bounds(poly: MultiPolygon<f64>) -> geo::Rect<f64> {
    use geo::bounding_rect::BoundingRect;
    poly.bounding_rect().unwrap()
}

impl<U, S> fmt::Display for AtPos<U, S>
where
    U: super::Feature + std::fmt::Debug + Clone,
    S: super::Feature + std::fmt::Debug + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "pos(U = {}, S = {:?})", self.inner, self.elements,)
    }
}

impl<U, S> super::Feature for AtPos<U, S>
where
    U: super::Feature + std::fmt::Debug + Clone,
    S: super::Feature + std::fmt::Debug + Clone,
{
    fn name(&self) -> &'static str {
        "pos"
    }

    fn edge_union(&self) -> Option<MultiPolygon<f64>> {
        use geo::algorithm::translate::Translate;
        use geo_booleanop::boolean::BooleanOp;

        let mut out = match self.inner.edge_union() {
            Some(p) => p,
            None => MultiPolygon(vec![]),
        };
        let bounds = compute_bounds(out.clone());

        for (feature, position) in &self.elements {
            if let Some(mut geo) = feature.edge_union() {
                let t = position.compute_translation(bounds, compute_bounds(geo.clone()));
                geo.translate_inplace(t.0, t.1);
                out = out.union(&geo)
            }
        }

        if out.0.len() > 0 {
            Some(out)
        } else {
            None
        }
    }

    fn translate(&mut self, v: Coordinate<f64>) {
        self.inner.translate(v);
        // No need to move the others, we position them ourselves.
    }

    fn interior(&self) -> Vec<super::InnerAtom> {
        let bounds = compute_bounds(match self.inner.edge_union() {
            Some(p) => p,
            None => MultiPolygon(vec![]),
        });

        self.inner
            .interior()
            .into_iter()
            .chain(
                self.elements
                    .iter()
                    .map(|(feature, position)| {
                        if let Some(geo) = feature.edge_union() {
                            let t =
                                position.compute_translation(bounds, compute_bounds(geo.clone()));
                            let mut out = feature.interior();
                            for a in out.iter_mut() {
                                a.translate(t.0, t.1);
                            }
                            out
                        } else {
                            vec![]
                        }
                    })
                    .flatten(),
            )
            .collect()
    }
}
