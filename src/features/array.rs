use geo::{Coordinate, MultiPolygon};
use std::fmt;

/// A feature which aligns a sequence of features vertically.
#[derive(Debug, Clone)]
pub struct Column<U = super::Unit> {
    array: Vec<U>,
    align: crate::Align,
    bbox: bool,
}

impl<U: super::Feature + fmt::Debug + Clone> Column<U> {
    /// Lays out the given features in an array going downwards, with
    /// their leftmost elements aligned.
    pub fn align_left(array: Vec<U>) -> Self {
        Self::new(array, crate::Align::Start)
    }

    /// Lays out the given features in an array going downwards, with
    /// their rightmost elements aligned.``
    pub fn align_right(array: Vec<U>) -> Self {
        Self::new(array, crate::Align::End)
    }

    /// Lays out the given features in an array going downwards, with
    /// each element aligned to the center.
    pub fn align_center(array: Vec<U>) -> Self {
        Self::new(array, crate::Align::Center)
    }

    fn new(mut array: Vec<U>, align: crate::Align) -> Self {
        // Position any containing geometry to exist entirely in positive
        // (x>=0, y>=0) coordinate space.
        for e in array.iter_mut() {
            if let Some(b) = e.edge_union() {
                use geo::bounding_rect::BoundingRect;
                let v = b.bounding_rect().unwrap().min();
                e.translate(-v);
            }
        }

        Self {
            align,
            array,
            bbox: true,
        }
    }

    fn all_bounds(&self) -> Vec<geo::Rect<f64>> {
        self.array
            .iter()
            .map(|f| match f.edge_union() {
                Some(edge) => {
                    use geo::bounding_rect::BoundingRect;
                    edge.bounding_rect().unwrap()
                }
                None => geo::Rect::new(
                    Coordinate::<f64> { x: 0., y: 0. },
                    Coordinate::<f64> { x: 0., y: 0. },
                ),
            })
            .collect()
    }

    fn largest(&self) -> geo::Rect<f64> {
        self.all_bounds()
            .into_iter()
            .max_by(|x, y| x.width().partial_cmp(&y.width()).unwrap())
            .unwrap()
    }

    fn translations<'a>(
        &'a self,
        largest: geo::Rect<f64>,
    ) -> Box<dyn Iterator<Item = Option<(f64, f64)>> + 'a> {
        Box::new(
            self.array
                .iter()
                .map(|f| match f.edge_union() {
                    Some(edge_geo) => Some(edge_geo.clone()),
                    None => None,
                })
                .scan(0f64, |y_off, f| {
                    // accumulate the heights of each element so we can
                    // adjust them to tile downwards.
                    use geo::bounding_rect::BoundingRect;
                    let h = match f {
                        Some(ref f) => f.clone().bounding_rect().unwrap().height(),
                        None => 0.0,
                    };
                    let out = Some((f, *y_off));
                    *y_off = *y_off + h;
                    out
                })
                .map(move |(g, y_off)| match g {
                    Some(g) => {
                        use geo::bounding_rect::BoundingRect;
                        let bounds = g.bounding_rect().unwrap();

                        Some(match self.align {
                            crate::Align::Start => (largest.min().x - bounds.min().x, y_off),
                            crate::Align::End => (largest.max().x - bounds.max().x, y_off),
                            crate::Align::Center => (largest.center().x - bounds.center().x, y_off),
                        })
                    }
                    None => None,
                }),
        )
    }
}

impl<U: super::Feature + fmt::Debug> fmt::Display for Column<U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Column(align = {:?}, {:?})", self.align, self.array)
    }
}

impl<U: super::Feature + fmt::Debug + Clone> super::Feature for Column<U> {
    fn name(&self) -> &'static str {
        "Column"
    }

    fn edge_union(&self) -> Option<MultiPolygon<f64>> {
        let out = self
            .array
            .iter()
            .map(|f| match f.edge_union() {
                Some(edge_geo) => Some(edge_geo.clone()),
                None => None,
            })
            .zip(self.translations(self.largest()).into_iter())
            .filter(|(f, t)| f.is_some() && t.is_some())
            .map(|(f, t)| (f.unwrap(), t.unwrap()))
            .fold(None, |mut acc, (g, (tx, ty))| {
                use geo::translate::Translate;
                use geo_booleanop::boolean::BooleanOp;
                if let Some(current) = acc {
                    acc = Some(g.translate(tx, ty).union(&current));
                } else {
                    acc = Some(g.translate(tx, ty));
                };
                acc
            });

        // If we are in bbox mode, all we need to do is compute the bounding
        // box and use that as our outer geometry.
        if self.bbox {
            match out {
                None => None,
                Some(poly) => {
                    use geo::bounding_rect::BoundingRect;
                    Some(poly.bounding_rect().unwrap().to_polygon().into())
                }
            }
        } else {
            out
        }
    }

    fn translate(&mut self, v: Coordinate<f64>) {
        for e in self.array.iter_mut() {
            e.translate(v);
        }
    }

    fn interior(&self) -> Vec<super::InnerAtom> {
        let largest = self.largest();

        self.array
            .iter()
            .map(|f| f.interior())
            .zip(self.translations(largest).into_iter())
            .map(|(f, t)| {
                let (tx, ty) = match t {
                    Some((tx, ty)) => (tx, ty),
                    None => (0., 0.),
                };

                f.into_iter().map(move |mut a| {
                    a.translate(tx, ty);
                    a
                })
            })
            .flatten()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::Rect;
    use test_case::test_case;

    #[test]
    fn bounds() {
        let a = Column::align_left(vec![
            Rect::with_center([0., 0.].into(), 2., 3.),
            Rect::with_center([0., 0.].into(), 3., 2.),
        ]);

        assert_eq!(
            a.all_bounds(),
            vec![
                geo::Rect::<f64>::new::<geo::Coordinate<_>>([0., 0.].into(), [2., 3.].into()),
                geo::Rect::<f64>::new::<geo::Coordinate<_>>([0., 0.].into(), [3., 2.].into()),
            ]
        );
    }

    #[test]
    fn largest() {
        let a = Column::align_left(vec![
            Rect::with_center([0., 0.].into(), 2., 3.),
            Rect::with_center([0., 0.].into(), 3., 2.),
        ]);

        assert_eq!(
            a.largest(),
            geo::Rect::<f64>::new::<geo::Coordinate<_>>([0., 0.].into(), [3., 2.].into(),),
        );
    }

    #[test_case(
        vec![
            Rect::with_center([0., 0.].into(), 4., 2.),
            Rect::with_center([0., 0.].into(), 2., 2.),
        ], vec![
            Some((0., 0.)),
            Some((0., 2.)),
        ] ; "origin centered"
    )
    ]
    fn translations_left(inners: Vec<Rect>, want: Vec<Option<(f64, f64)>>) {
        let a = Column::align_left(inners);
        assert_eq!(a.translations(a.largest()).collect::<Vec<_>>(), want,);
    }

    #[test_case( vec![
        Rect::with_center([0., 0.].into(), 2., 2.),
        Rect::with_center([0., 0.].into(), 4., 2.),
    ], vec![
        Some((1., 0.)),
        Some((0., 2.)),
    ] ; "origin centered")]
    #[test_case( vec![
        Rect::new([0., 0.].into(), [2., 3.].into()),
        Rect::new([0., 0.].into(), [2., 2.].into()),
    ], vec![
        Some((0., 0.)),
        Some((0., 3.)),
    ] ; "origin tl zeroed")]
    fn translations_center(inners: Vec<Rect>, want: Vec<Option<(f64, f64)>>) {
        let a = Column::align_center(inners);
        assert_eq!(a.translations(a.largest()).collect::<Vec<_>>(), want,);
    }
}
