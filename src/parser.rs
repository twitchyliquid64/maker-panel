use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_while};
use nom::character::complete::multispace0;
use nom::combinator::{map, opt};
use nom::multi::{fold_many1, many0};
use nom::sequence::{delimited, tuple};
use nom::IResult;

#[derive(Debug, Clone)]
pub enum InnerAST {
    ScrewHole(f64),
}

impl InnerAST {
    fn into_inner_feature<'a>(self) -> Box<dyn super::features::InnerFeature + 'a> {
        use super::features::ScrewHole;

        match self {
            InnerAST::ScrewHole(dia) => Box::new(ScrewHole::with_diameter(dia)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AST {
    Rect {
        coords: Option<(f64, f64)>,
        size: Option<(f64, f64)>,
        inner: Option<InnerAST>,
    },
    Circle {
        coords: Option<(f64, f64)>,
        radius: f64,
        inner: Option<InnerAST>,
    },
    Array {
        num: usize,
        inner: Box<AST>,
    },
}

impl AST {
    fn into_feature<'a>(self) -> Box<dyn super::Feature + 'a> {
        use super::features::{Circle, Rect};

        match self {
            AST::Rect {
                coords,
                size,
                inner,
            } => {
                if let Some(inner) = inner {
                    let r = Rect::with_inner(inner.into_inner_feature());
                    let (w, h) = if let Some((w, h)) = size {
                        (w, h)
                    } else {
                        (2., 2.)
                    };
                    let r = if let Some(coords) = coords {
                        r.dimensions(coords.into(), w, h)
                    } else {
                        r.dimensions([0., 0.].into(), w, h)
                    };
                    Box::new(r)
                } else {
                    Box::new(match (coords, size) {
                        (Some(c), Some(s)) => Rect::with_center(c.into(), s.0, s.1),
                        (None, Some(s)) => Rect::with_center([0., 0.].into(), s.0, s.1),
                        (Some(c), None) => Rect::with_center(c.into(), 2., 2.),
                        (None, None) => Rect::with_center([-1f64, -1f64].into(), 2., 2.),
                    })
                }
            }
            AST::Circle {
                coords,
                radius,
                inner,
            } => match (inner, coords) {
                (Some(i), Some(c)) => {
                    Box::new(Circle::with_inner(i.into_inner_feature(), c.into(), radius))
                }
                (Some(i), None) => {
                    Box::new(Circle::wrap_with_radius(i.into_inner_feature(), radius))
                }
                (None, Some(c)) => Box::new(Circle::new(c.into(), radius)),
                (None, None) => Box::new(Circle::with_radius(radius)),
            },
            AST::Array { num, inner } => Box::new(crate::features::repeating::Tile::new(
                inner.into_feature(),
                crate::Direction::Right,
                num,
            )),
        }
    }
}

fn parse_uint(i: &str) -> IResult<&str, usize> {
    let (i, _) = multispace0(i)?;
    let (i, s) = take_while(|c| c == '-' || (c >= '0' && c <= '9'))(i)?;
    Ok((
        i,
        s.parse().map_err(|_e| {
            nom::Err::Error(nom::error::Error::new(i, nom::error::ErrorKind::Digit))
        })?,
    ))
}

fn parse_float(i: &str) -> IResult<&str, f64> {
    let (i, _) = multispace0(i)?;
    let (i, s) = take_while(|c| c == '.' || c == '-' || (c >= '0' && c <= '9'))(i)?;
    Ok((
        i,
        s.parse().map_err(|_e| {
            nom::Err::Error(nom::error::Error::new(i, nom::error::ErrorKind::Digit))
        })?,
    ))
}

fn parse_coords(i: &str) -> IResult<&str, (f64, f64)> {
    let (i, _) = multispace0(i)?;
    let (i, _) = tag("(")(i)?;
    let (i, x) = parse_float(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag(",")(i)?;
    let (i, y) = parse_float(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag(")")(i)?;
    Ok((i, (x, y)))
}

fn parse_inner(i: &str) -> IResult<&str, InnerAST> {
    let (i, _) = multispace0(i)?;

    let (i, inner) = delimited(
        tuple((tag("("), multispace0)),
        alt((
            map(tuple((tag("h"), parse_float)), |(_, f)| {
                InnerAST::ScrewHole(f)
            }),
            map(tag("h"), |_| InnerAST::ScrewHole(3.1)),
        )),
        tuple((multispace0, tag(")"))),
    )(i)?;

    Ok((i, inner))
}

enum DetailFragment {
    Coord(f64, f64),
    Size(f64, f64),
    Radius(f64),
    Extra(f64),
}

#[derive(Debug, Default, Clone)]
struct Details {
    coords: Option<(f64, f64)>,
    size: Option<(f64, f64)>,
    radius: Option<f64>,
    extra: Vec<f64>,
    inner: Option<InnerAST>,
}

impl Details {
    fn parse_pos(i: &str) -> IResult<&str, DetailFragment> {
        let (i, _) = multispace0(i)?;
        let (i, _t) = tag("@")(i)?;
        let (i, c) = parse_coords(i)?;
        Ok((i, DetailFragment::Coord(c.0, c.1)))
    }
    fn parse_extra(i: &str) -> IResult<&str, DetailFragment> {
        let (i, f) = parse_float(i)?;
        Ok((i, DetailFragment::Extra(f)))
    }
    fn parse_size(i: &str) -> IResult<&str, DetailFragment> {
        let (i, _) = multispace0(i)?;
        let (i, (_, _, _, _, c)) = tuple((
            alt((tag_no_case("size"), tag_no_case("s"))),
            multispace0,
            tag("="),
            multispace0,
            parse_coords,
        ))(i)?;
        Ok((i, DetailFragment::Size(c.0, c.1)))
    }
    fn parse_radius(i: &str) -> IResult<&str, DetailFragment> {
        let (i, _) = multispace0(i)?;
        let (i, (_, _, _, _, r)) = tuple((
            alt((tag_no_case("radius"), tag_no_case("r"))),
            multispace0,
            tag("="),
            multispace0,
            parse_float,
        ))(i)?;
        Ok((i, DetailFragment::Radius(r)))
    }

    fn with_inner(mut self, inner: Option<InnerAST>) -> Self {
        self.inner = inner;
        self
    }
}

fn parse_details(i: &str) -> IResult<&str, Details> {
    let (i, _) = multispace0(i)?;

    let (i, deets) = delimited(
        tuple((tag("<"), multispace0)),
        fold_many1(
            alt((
                tuple((Details::parse_pos, multispace0, opt(tag(",")))),
                tuple((Details::parse_size, multispace0, opt(tag(",")))),
                tuple((Details::parse_radius, multispace0, opt(tag(",")))),
                tuple((Details::parse_extra, multispace0, opt(tag(",")))),
            )),
            Details::default(),
            |mut acc: Details, (fragment, _, _)| {
                match fragment {
                    DetailFragment::Coord(x, y) => {
                        acc.coords = Some((x, y));
                    }
                    DetailFragment::Size(x, y) => {
                        acc.size = Some((x, y));
                    }
                    DetailFragment::Radius(r) => {
                        acc.radius = Some(r);
                    }
                    DetailFragment::Extra(f) => acc.extra.push(f),
                }
                acc
            },
        ),
        tuple((tag(">"), multispace0)),
    )(i)?;

    let (i, inner) = opt(parse_inner)(i)?;
    Ok((i, deets.with_inner(inner)))
}

fn parse_rect(i: &str) -> IResult<&str, AST> {
    let (i, _) = multispace0(i)?;
    let (i, _) = tag_no_case("R")(i)?;
    let (i, deets) = parse_details(i)?;

    let size = if let Some((x, y)) = deets.size {
        Some((x, y))
    } else if deets.extra.len() == 2 {
        Some((deets.extra[0], deets.extra[1]))
    } else if deets.extra.len() == 1 {
        Some((deets.extra[0], deets.extra[0]))
    } else {
        None
    };

    Ok((
        i,
        AST::Rect {
            size,
            coords: deets.coords,
            inner: deets.inner,
        },
    ))
}

fn parse_circle(i: &str) -> IResult<&str, AST> {
    let (i, _) = multispace0(i)?;
    let (i, _) = tag_no_case("C")(i)?;
    let (i2, deets) = parse_details(i)?;

    let r = if let Some(r) = deets.radius {
        r
    } else if deets.extra.len() == 1 {
        deets.extra[0]
    } else {
        return Err(nom::Err::Failure(nom::error::make_error(
            i,
            nom::error::ErrorKind::Satisfy,
        )));
    };

    Ok((
        i2,
        AST::Circle {
            coords: deets.coords,
            radius: r,
            inner: deets.inner,
        },
    ))
}

fn parse_array(i: &str) -> IResult<&str, AST> {
    let (i, _) = multispace0(i)?;

    let (i, num) = delimited(
        tuple((tag("["), multispace0)),
        parse_uint,
        tuple((tag("]"), multispace0)),
    )(i)?;
    let (i, geo) = parse_geo(i)?;

    Ok((
        i,
        AST::Array {
            num,
            inner: Box::new(geo),
        },
    ))
}

fn parse_geo(i: &str) -> IResult<&str, AST> {
    alt((parse_array, parse_rect, parse_circle))(i)
}

/// Parses the provided panel spec and returns the series of features
/// it represents.
pub fn build<'a>(i: &str) -> Result<Vec<Box<dyn super::Feature + 'a>>, ()> {
    Ok(many0(parse_geo)(i)
        .map_err(|_e| ())?
        .1
        .into_iter()
        .map(|g| g.into_feature())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect() {
        let out = parse_geo("R<@(1,2)>");
        assert!(
            matches!(out, Ok(("", AST::Rect{ coords: Some(c), size: None, inner: _ })) if
                c.0 > 0.99 && c.0 < 1.01 && c.1 > 1.99 && c.1 < 2.01
            )
        );

        let out = parse_geo("R<@(1,2), 2, 4>");
        assert!(
            matches!(out, Ok(("", AST::Rect{ coords: Some(c), size: Some((w, h)), inner: _ })) if
                c.0 > 0.99 && c.0 < 1.01 && c.1 > 1.99 && c.1 < 2.01 &&
                w > 1.99 && w < 2.01 && h > 3.99 && h < 4.01
            )
        );

        let out = parse_geo("R<@(1,2), 4>");
        assert!(
            matches!(out, Ok(("", AST::Rect{ coords: Some(c), size: Some((w, h)), inner: _ })) if
                c.0 > 0.99 && c.0 < 1.01 && c.1 > 1.99 && c.1 < 2.01 &&
                w > 3.99 && w < 4.01 && h > 3.99 && h < 4.01
            )
        );

        let out = parse_geo("R<4>");
        assert!(
            matches!(out, Ok(("", AST::Rect{ coords: None, size: Some((w, h)), inner: _ })) if
                w > 3.99 && w < 4.01 && h > 3.99 && h < 4.01
            )
        );

        let out = parse_geo("R<@(1,2), size = (2,4)>");
        assert!(
            matches!(out, Ok(("", AST::Rect{ coords: Some(c), size: Some((w, h)), inner: _ })) if
                c.0 > 0.99 && c.0 < 1.01 && c.1 > 1.99 && c.1 < 2.01 &&
                w > 1.99 && w < 2.01 && h > 3.99 && h < 4.01
            )
        );

        let out = parse_geo("R<6>(h)");
        assert!(
            matches!(out, Ok(("", AST::Rect{ coords: None, size: Some((w, h)), inner: Some(InnerAST::ScrewHole(dia)) })) if
                w > 5.99 && w < 6.01 && h > 5.99 && h < 6.01 &&
                dia < 3.11 && dia > 3.09
            )
        );
    }

    #[test]
    fn test_circle() {
        let out = parse_geo("C < @ ( 2 , 1 ), 4.5>");
        assert!(
            matches!(out, Ok(("", AST::Circle{ coords: Some(c), radius: r, inner: _ })) if
                c.1 > 0.99 && c.1 < 1.01 && c.0 > 1.99 && c.0 < 2.01 &&
                r > 4.49 && r < 4.51
            )
        );

        let out = parse_geo("C<@(2, 1), 3.5>");
        assert!(
            matches!(out, Ok(("", AST::Circle{ coords: Some(c), radius: r, inner: _ })) if
                c.1 > 0.99 && c.1 < 1.01 && c.0 > 1.99 && c.0 < 2.01 &&
                r > 3.49 && r < 3.51
            )
        );
        let out = parse_geo("C<3.5>");
        assert!(
            matches!(out, Ok(("", AST::Circle{ coords: None, radius: r, inner: _ })) if
                r > 3.49 && r < 3.51
            )
        );

        let out = parse_geo("C<@(2, 1), R=3.5>");
        assert!(
            matches!(out, Ok(("", AST::Circle{ coords: Some(c), radius: r, inner: _ })) if
                c.1 > 0.99 && c.1 < 1.01 && c.0 > 1.99 && c.0 < 2.01 &&
                r > 3.49 && r < 3.51
            )
        );

        let out = parse_geo("C<3.5> ( h9 )");
        assert!(
            matches!(out, Ok(("", AST::Circle{ coords: None, radius: r, inner: Some(InnerAST::ScrewHole(dia)) })) if
                r > 3.49 && r < 3.51 && dia > 8.999 && dia < 9.001
            )
        );
    }

    #[test]
    fn test_array() {
        let out = parse_geo("[5]C<4.5>");
        eprintln!("{:?}", out);
        assert!(matches!(out, Ok(("", AST::Array{ num: 5, inner: b})) if
            matches!(*b, AST::Circle{ radius, .. } if radius > 4.4 && radius < 4.6)
        ));
    }
}
