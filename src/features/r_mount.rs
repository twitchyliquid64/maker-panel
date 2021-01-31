use super::InnerAtom;
use geo::{Coordinate, MultiPolygon};
use std::fmt;

/// A cut-out edge region for mounting a board at right angles.
#[derive(Debug, Clone)]
pub struct RMount {
    direction: crate::Direction,
    rect: geo::Rect<f64>,
    depth: f64,
}

impl RMount {
    /// Creates a new r-mount with the provided depth.
    pub fn new(depth: f64) -> Self {
        let direction = crate::Direction::Up;
        let tl: Coordinate<f64> = [-3.15f64, -depth / 2. - 1.].into();
        let br: Coordinate<f64> = [3.15f64, depth / 2. + 1.].into();
        let rect = geo::Rect::new(tl, br);
        Self {
            direction,
            rect,
            depth,
        }
    }

    /// Returns a new r-mount with the specified direction.
    pub fn direction(self, direction: crate::Direction) -> Self {
        Self { direction, ..self }
    }
}

impl fmt::Display for RMount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rmount({:?}, {:?})", self.rect.min(), self.rect.max(),)
    }
}

impl super::Feature for RMount {
    fn name(&self) -> &'static str {
        "rmount"
    }

    fn edge_union(&self) -> Option<MultiPolygon<f64>> {
        let out = self.rect.clone().to_polygon().into();
        use geo::algorithm::rotate::Rotate;
        match self.direction {
            crate::Direction::Up => Some(out),
            crate::Direction::Down => Some(out.rotate(180.)),
            crate::Direction::Left => Some(out.rotate(-90.)),
            crate::Direction::Right => Some(out.rotate(90.)),
        }
    }

    fn edge_subtract(&self) -> Option<MultiPolygon<f64>> {
        use geo_booleanop::boolean::BooleanOp;
        let center = self.rect.center();

        let channel = geo::Rect::new(
            Coordinate::<f64>::from([-1.65f64, -self.depth / 2. - 1.]) + center,
            Coordinate::<f64>::from([1.65f64, self.depth / 2.]) + center,
        );
        let nut = geo::Rect::new(
            Coordinate::<f64>::from([-3.12f64, -1.4]) + center,
            Coordinate::<f64>::from([3.12f64, 1.4]) + center,
        );

        let out = MultiPolygon::<f64>::from(channel.to_polygon()).union(&nut.to_polygon());

        use geo::algorithm::rotate::Rotate;
        match self.direction {
            crate::Direction::Up => Some(out),
            crate::Direction::Down => Some(out.rotate(180.)),
            crate::Direction::Left => Some(out.rotate(-90.)),
            crate::Direction::Right => Some(out.rotate(90.)),
        }
    }

    fn translate(&mut self, v: Coordinate<f64>) {
        use geo::algorithm::translate::Translate;
        self.rect.translate_inplace(v.x, v.y);
    }

    fn interior(&self) -> Vec<super::InnerAtom> {
        let center = self.rect.center();
        let angle = match self.direction {
            crate::Direction::Up => 0.,
            crate::Direction::Down => 180.,
            crate::Direction::Left => -90.,
            crate::Direction::Right => 90.,
        };
        let origin = geo::Point::new(0., 0.);

        use geo::algorithm::rotate::RotatePoint;
        vec![
            InnerAtom::Drill {
                center: (geo::Point::from([-3.08f64, -1.32]).rotate_around_point(angle, origin)
                    + center.into())
                .into(),
                radius: 0.15,
                plated: false,
            },
            InnerAtom::Drill {
                center: (geo::Point::from([3.08f64, -1.32]).rotate_around_point(angle, origin)
                    + center.into())
                .into(),
                radius: 0.15,
                plated: false,
            },
            InnerAtom::Drill {
                center: (geo::Point::from([-3.08f64, 1.32]).rotate_around_point(angle, origin)
                    + center.into())
                .into(),
                radius: 0.15,
                plated: false,
            },
            InnerAtom::Drill {
                center: (geo::Point::from([3.08f64, 1.32]).rotate_around_point(angle, origin)
                    + center.into())
                .into(),
                radius: 0.15,
                plated: false,
            },
        ]
    }
}
