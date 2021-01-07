use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_while};
use nom::character::{
    complete::{multispace0, none_of, one_of},
    is_digit,
};
use nom::combinator::opt;
use nom::multi::{fold_many1, many0, separated_list1};
use nom::sequence::{delimited, tuple};
use nom::IResult;

#[derive(Debug, Clone)]
pub enum AST {
    Rect {
        coords: Option<(f64, f64)>,
        size: Option<(f64, f64)>,
    },
    Circle {
        coords: Option<(f64, f64)>,
        radius: f64,
    },
}

fn parse_float(i: &str) -> IResult<&str, f64> {
    let (i, _) = multispace0(i)?;
    let (i, s) = take_while(|c| c == '.' || (c >= '0' && c <= '9'))(i)?;
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

    Ok((i, deets))
}

fn parse_rect(i: &str) -> IResult<&str, AST> {
    let (i, _) = tag_no_case("R")(i)?;
    let (i, deets) = parse_details(i)?;

    let size = if let Some((x, y)) = deets.size {
        Some((x, y))
    } else if deets.extra.len() == 2 {
        Some((deets.extra[0], deets.extra[1]))
    } else {
        None
    };

    Ok((
        i,
        AST::Rect {
            size,
            coords: deets.coords,
        },
    ))
}

fn parse_circle(i: &str) -> IResult<&str, AST> {
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
        },
    ))
}

pub fn parse_geo(i: &str) -> IResult<&str, AST> {
    alt((parse_rect, parse_circle))(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect() {
        let out = parse_geo("R<@(1,2)>");
        assert!(
            matches!(out, Ok(("", AST::Rect{ coords: Some(c), size: None })) if
                c.0 > 0.99 && c.0 < 1.01 && c.1 > 1.99 && c.1 < 2.01
            )
        );

        let out = parse_geo("R<@(1,2), 2, 4>");
        assert!(
            matches!(out, Ok(("", AST::Rect{ coords: Some(c), size: Some((w, h)) })) if
                c.0 > 0.99 && c.0 < 1.01 && c.1 > 1.99 && c.1 < 2.01 &&
                w > 1.99 && w < 2.01 && h > 3.99 && h < 4.01
            )
        );

        let out = parse_geo("R<@(1,2), size = (2,4)>");
        assert!(
            matches!(out, Ok(("", AST::Rect{ coords: Some(c), size: Some((w, h)) })) if
                c.0 > 0.99 && c.0 < 1.01 && c.1 > 1.99 && c.1 < 2.01 &&
                w > 1.99 && w < 2.01 && h > 3.99 && h < 4.01
            )
        );
    }

    #[test]
    fn test_circle() {
        let out = parse_geo("C < @ ( 2 , 1 ), 4.5>");
        assert!(
            matches!(out, Ok(("", AST::Circle{ coords: Some(c), radius: r })) if
                c.1 > 0.99 && c.1 < 1.01 && c.0 > 1.99 && c.0 < 2.01 &&
                r > 4.49 && r < 4.51
            )
        );

        let out = parse_geo("C<@(2, 1), 3.5>");
        assert!(
            matches!(out, Ok(("", AST::Circle{ coords: Some(c), radius: r })) if
                c.1 > 0.99 && c.1 < 1.01 && c.0 > 1.99 && c.0 < 2.01 &&
                r > 3.49 && r < 3.51
            )
        );

        let out = parse_geo("C<@(2, 1), R=3.5>");
        assert!(
            matches!(out, Ok(("", AST::Circle{ coords: Some(c), radius: r })) if
                c.1 > 0.99 && c.1 < 1.01 && c.0 > 1.99 && c.0 < 2.01 &&
                r > 3.49 && r < 3.51
            )
        );
    }
}
