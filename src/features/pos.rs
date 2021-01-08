use geo::{Coordinate, MultiPolygon};
use std::fmt;

/// A wrapper around a feature that can position other features.
#[derive(Debug, Clone)]
pub struct AtPos<U = super::Unit, L = super::Unit, R = super::Unit> {
    inner: U,
    left: Option<L>,
    right: Option<R>,
}

impl<U, L, R> AtPos<U, L, R>
where
    U: super::Feature + std::fmt::Debug + Clone,
    L: super::Feature + std::fmt::Debug + Clone,
    R: super::Feature + std::fmt::Debug + Clone,
{
    /// Constructs a feature that positions the centeroid of other
    /// features at the left & right points of the primary feature.
    pub fn x_ends(primary: U, left: Option<L>, right: Option<R>) -> Self {
        Self {
            inner: primary,
            left,
            right,
        }
    }
}

impl<U, L> AtPos<U, L, super::Unit>
where
    U: super::Feature + std::fmt::Debug + Clone,
    L: super::Feature + std::fmt::Debug + Clone,
{
    /// Constructs a feature that positions the centeroid of the other
    /// feature to the left of the primary feature.
    pub fn left(primary: U, left: Option<L>) -> Self {
        Self {
            left,
            inner: primary,
            right: None,
        }
    }
}

impl<U, R> AtPos<U, super::Unit, R>
where
    U: super::Feature + std::fmt::Debug + Clone,
    R: super::Feature + std::fmt::Debug + Clone,
{
    /// Constructs a feature that positions the centeroid of the other
    /// feature to the right of the primary feature.
    pub fn right(primary: U, right: Option<R>) -> Self {
        Self {
            right,
            inner: primary,
            left: None,
        }
    }
}

fn compute_bounds(poly: MultiPolygon<f64>) -> geo::Rect<f64> {
    use geo::bounding_rect::BoundingRect;
    poly.bounding_rect().unwrap()
}

fn compute_left_translation(bounds: geo::Rect<f64>, lb: geo::Rect<f64>) -> (f64, f64) {
    (
        bounds.min().x - lb.center().x,
        bounds.center().y - lb.center().y,
    )
}

fn compute_right_translation(bounds: geo::Rect<f64>, rb: geo::Rect<f64>) -> (f64, f64) {
    (
        bounds.max().x - rb.center().x,
        bounds.center().y - rb.center().y,
    )
}

impl<U, L, R> fmt::Display for AtPos<U, L, R>
where
    U: super::Feature + std::fmt::Debug + Clone,
    L: super::Feature + std::fmt::Debug + Clone,
    R: super::Feature + std::fmt::Debug + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "pos(U = {}, L = {:?}, R = {:?})",
            self.inner, self.left, self.right,
        )
    }
}

impl<U, L, R> super::Feature for AtPos<U, L, R>
where
    U: super::Feature + std::fmt::Debug + Clone,
    L: super::Feature + std::fmt::Debug + Clone,
    R: super::Feature + std::fmt::Debug + Clone,
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

        if let Some(left) = &self.left {
            if let Some(mut left) = left.edge_union() {
                let t = compute_left_translation(bounds, compute_bounds(left.clone()));
                left.translate_inplace(t.0, t.1);
                out = out.union(&left)
            }
        }
        if let Some(right) = &self.right {
            if let Some(mut right) = right.edge_union() {
                let t = compute_right_translation(bounds, compute_bounds(right.clone()));
                right.translate_inplace(t.0, t.1);
                out = out.union(&right)
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
                if let Some(left) = &self.left {
                    if let Some(left_edges) = left.edge_union() {
                        let t =
                            compute_left_translation(bounds, compute_bounds(left_edges.clone()));
                        let mut out = left.interior();
                        for a in out.iter_mut() {
                            a.translate(t.0, t.1);
                        }
                        out
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
                .into_iter(),
            )
            .chain(
                if let Some(right) = &self.right {
                    if let Some(right_edges) = right.edge_union() {
                        let t =
                            compute_right_translation(bounds, compute_bounds(right_edges.clone()));
                        let mut out = right.interior();
                        for a in out.iter_mut() {
                            a.translate(t.0, t.1);
                        }
                        out
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
                .into_iter(),
            )
            .collect()
    }
}
