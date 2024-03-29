use crate::Direction;
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_while};
use nom::character::complete::{multispace0, one_of};
use nom::combinator::{all_consuming, cut, map, opt};
use nom::error::{context, VerboseError};
use nom::multi::{fold_many1, many0};
use nom::sequence::{delimited, tuple};
use nom::IResult;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    Float(f64),
    Ref(String),
    Cel(String),
}

impl Value {
    fn float(&self) -> f64 {
        match self {
            Value::Float(f) => *f,
            _ => unimplemented!(),
        }
    }

    fn rfloat(&self, r: &ResolverContext) -> Result<f64, Err> {
        match self {
            Value::Float(f) => Ok(*f),
            Value::Ref(ident) => match r.definitions.get(ident) {
                Some(var) => match var {
                    Variable::Number(n) => Ok(*n),
                    _ => Err(Err::BadType(ident.to_string())),
                },
                None => Err(Err::UndefinedVariable(ident.to_string())),
            },
            Value::Cel(exp) => {
                use cel_interpreter::*;
                match r.eval_cel(exp.to_string()) {
                    objects::CelType::UInt(n) => Ok(n as f64),
                    objects::CelType::Int(n) => Ok(n as f64),
                    objects::CelType::Float(n) => Ok(n),
                    _ => Err(Err::BadType(exp.to_string())),
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Variable {
    Geo(AST),
    Number(f64),
}

#[derive(Debug, Clone, Default)]
struct ResolverContext {
    pub definitions: HashMap<String, Variable>,
}

impl ResolverContext {
    fn handle_assignment(&mut self, var: String, ast: Box<AST>) {
        match *ast {
            AST::Cel(exp) => {
                use cel_interpreter::objects::CelType;
                match self.eval_cel(exp) {
                    CelType::UInt(n) => {
                        self.definitions.insert(var, Variable::Number(n as f64));
                    }
                    CelType::Int(n) => {
                        self.definitions.insert(var, Variable::Number(n as f64));
                    }
                    CelType::Float(n) => {
                        self.definitions.insert(var, Variable::Number(n));
                    }
                    _ => panic!(),
                }
            }
            _ => {
                self.definitions.insert(var, Variable::Geo(*ast));
            }
        }
    }

    fn cel_ctx(&self) -> cel_interpreter::context::Context {
        use cel_interpreter::*;
        let mut ctx = context::Context::default();
        for (ident, val) in &self.definitions {
            match val {
                Variable::Geo(_) => {}
                Variable::Number(n) => {
                    ctx.add_variable(ident.clone(), objects::CelType::Float(*n));
                }
            }
        }

        ctx
    }

    fn eval_cel(&self, exp: String) -> cel_interpreter::objects::CelType {
        use cel_interpreter::*;
        match Program::compile(&exp) {
            Ok(p) => {
                let ctx = self.cel_ctx();
                p.execute(&ctx)
            }
            Err(e) => panic!("{}", e), // Should never panic: we checked while parsing
        }
    }
}

#[derive(Debug, Clone)]
pub enum Err {
    Parse(String),
    UndefinedVariable(String),
    BadType(String),
}

#[derive(Debug, Clone)]
pub enum InnerAST {
    ScrewHole(Value),
    Smiley,
    MechanicalSolderPoint(Option<(Value, Value)>),
}

impl InnerAST {
    fn into_inner_feature<'a>(
        self,
        _ctx: &mut ResolverContext,
    ) -> Box<dyn super::features::InnerFeature + 'a> {
        use super::features::{MechanicalSolderPoint, ScrewHole, Smiley};

        match self {
            InnerAST::ScrewHole(dia) => Box::new(ScrewHole::with_diameter(dia.float())),
            InnerAST::Smiley => Box::new(Smiley::default()),
            InnerAST::MechanicalSolderPoint(sz) => Box::new(match sz {
                Some((x, y)) => MechanicalSolderPoint::with_size((x.float(), y.float())),
                None => MechanicalSolderPoint::default(),
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub enum WrapPosition {
    Cardinal {
        side: Direction,
        offset: Value,
        align: crate::Align,
    },
    Corner {
        side: Direction,
        opposite: bool,
        align: crate::Align,
    },
    Angle {
        angle: Value,
        offset: Value,
    },
}

impl WrapPosition {
    fn into_positioning(self, r: &ResolverContext) -> Result<crate::features::Positioning, Err> {
        match self {
            WrapPosition::Cardinal {
                side,
                offset,
                align,
            } => Ok(crate::features::Positioning::Cardinal {
                side,
                align,
                centerline_adjustment: offset.rfloat(r)?,
            }),
            WrapPosition::Corner {
                side,
                opposite,
                align,
            } => Ok(crate::features::Positioning::Corner {
                side,
                align,
                opposite,
            }),
            WrapPosition::Angle { angle, offset } => Ok(crate::features::Positioning::Angle {
                degrees: angle.rfloat(r)?,
                amount: offset.rfloat(r)?,
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AST {
    Assign(String, Box<AST>),
    VarRef(String),
    Comment(String),
    Cel(String),
    Rect {
        coords: Option<(Value, Value)>,
        size: Option<(Value, Value)>,
        inner: Option<InnerAST>,
        rounded: Option<Value>,
    },
    Circle {
        coords: Option<(Value, Value)>,
        radius: Value,
        inner: Option<InnerAST>,
    },
    Triangle {
        size: (Value, Value),
        inner: Option<InnerAST>,
    },
    RMount {
        depth: Value,
        dir: crate::Direction,
    },
    Array {
        dir: crate::Direction,
        num: usize,
        inner: Box<AST>,
        vscore: bool,
    },
    ColumnLayout {
        coords: Option<(Value, Value)>,
        align: crate::Align,
        inners: Vec<Box<AST>>,
    },
    Wrap {
        inner: Box<AST>,
        features: Vec<(WrapPosition, Box<AST>)>,
    },
    Tuple {
        inners: Vec<Box<AST>>,
    },
    Negative {
        inners: Vec<Box<AST>>,
    },
    Rotate {
        rotation: Value,
        inners: Vec<Box<AST>>,
    },
    Name {
        name: String,
        inner: Box<AST>,
    },
}

impl AST {
    fn into_feature<'a>(
        self,
        ctx: &mut ResolverContext,
    ) -> Result<Box<dyn super::Feature + 'a>, Err> {
        use super::features::{Circle, RMount, Rect, Triangle};

        match self {
            AST::Rect {
                coords,
                size,
                inner,
                rounded: _,
            } => Ok(if let Some(inner) = inner {
                let r = Rect::with_inner(inner.into_inner_feature(ctx));
                let (w, h) = if let Some((w, h)) = size {
                    (w.rfloat(ctx)?, h.rfloat(ctx)?)
                } else {
                    (2., 2.)
                };
                let r = if let Some((x, y)) = coords {
                    r.dimensions((x.rfloat(ctx)?, y.rfloat(ctx)?).into(), w, h)
                } else {
                    r.dimensions([0., 0.].into(), w, h)
                };
                Box::new(r)
            } else {
                Box::new(match (coords, size) {
                    (Some((x, y)), Some((w, h))) => Rect::with_center(
                        (x.rfloat(ctx)?, y.rfloat(ctx)?).into(),
                        w.rfloat(ctx)?,
                        h.rfloat(ctx)?,
                    ),
                    (None, Some((w, h))) => {
                        Rect::with_center([0., 0.].into(), w.rfloat(ctx)?, h.rfloat(ctx)?)
                    }
                    (Some((x, y)), None) => {
                        Rect::with_center((x.rfloat(ctx)?, y.rfloat(ctx)?).into(), 2., 2.)
                    }
                    (None, None) => Rect::with_center([-1f64, -1f64].into(), 2., 2.),
                })
            }),
            AST::Circle {
                coords,
                radius,
                inner,
            } => Ok(match (inner, coords) {
                (Some(i), Some((x, y))) => Box::new(Circle::with_inner(
                    i.into_inner_feature(ctx),
                    (x.rfloat(ctx)?, y.rfloat(ctx)?).into(),
                    radius.rfloat(ctx)?,
                )),
                (Some(i), None) => Box::new(Circle::wrap_with_radius(
                    i.into_inner_feature(ctx),
                    radius.rfloat(ctx)?,
                )),
                (None, Some((x, y))) => Box::new(Circle::new(
                    (x.rfloat(ctx)?, y.rfloat(ctx)?).into(),
                    radius.rfloat(ctx)?,
                )),
                (None, None) => Box::new(Circle::with_radius(radius.rfloat(ctx)?)),
            }),
            AST::Triangle { size, inner } => Ok(match inner {
                Some(i) => Box::new(Triangle::with_inner(i.into_inner_feature(ctx)).dimensions(
                    [0., 0.].into(),
                    size.0.rfloat(ctx)?,
                    size.1.rfloat(ctx)?,
                )),
                None => Box::new(Triangle::right_angle(
                    size.0.rfloat(ctx)?,
                    size.1.rfloat(ctx)?,
                )),
            }),
            AST::RMount { depth, dir } => {
                Ok(Box::new(RMount::new(depth.rfloat(ctx)?).direction(dir)))
            }
            AST::Array {
                dir,
                num,
                inner,
                vscore,
            } => Ok(Box::new(
                crate::features::repeating::Tile::new(inner.into_feature(ctx)?, dir, num)
                    .v_score(vscore),
            )),
            AST::ColumnLayout {
                align,
                inners,
                coords,
            } => Ok(Box::new({
                let mut layout = match align {
                    crate::Align::Start => crate::features::Column::align_left(
                        inners
                            .into_iter()
                            .map(|i| i.into_feature(ctx))
                            .collect::<Result<Vec<_>, Err>>()?,
                    ),
                    crate::Align::Center => crate::features::Column::align_center(
                        inners
                            .into_iter()
                            .map(|i| i.into_feature(ctx))
                            .collect::<Result<Vec<_>, Err>>()?,
                    ),
                    crate::Align::End => crate::features::Column::align_right(
                        inners
                            .into_iter()
                            .map(|i| i.into_feature(ctx))
                            .collect::<Result<Vec<_>, Err>>()?,
                    ),
                };
                if let Some((x, y)) = coords {
                    use crate::features::Feature;
                    layout.translate([x.rfloat(ctx)?, y.rfloat(ctx)?].into());
                };
                layout
            })),
            AST::Wrap { inner, features } => {
                let mut pos = crate::features::AtPos::new(inner.into_feature(ctx)?);
                for (position, feature) in features {
                    pos.push(feature.into_feature(ctx)?, position.into_positioning(ctx)?);
                }
                Ok(Box::new(pos))
            }
            AST::Tuple { inners } => {
                let mut out: Option<Box<dyn super::Feature>> = None;
                for inner in inners.into_iter() {
                    out = match out {
                        None => Some(inner.into_feature(ctx)?),
                        Some(left) => Some(Box::new({
                            let mut out = crate::features::AtPos::new(left);
                            out.push(
                                inner.into_feature(ctx)?,
                                crate::features::Positioning::Cardinal {
                                    side: Direction::Right,
                                    align: crate::Align::End,
                                    centerline_adjustment: 0.,
                                },
                            );
                            out
                        })),
                    };
                }

                Ok(out.unwrap())
            }
            AST::Negative { inners } => Ok(Box::new(crate::features::Negative::new(
                inners
                    .into_iter()
                    .map(|f| f.into_feature(ctx))
                    .collect::<Result<Vec<_>, Err>>()?,
            ))),
            AST::Rotate { rotation, inners } => Ok(Box::new(crate::features::Rotate::new(
                rotation.rfloat(ctx)?,
                inners
                    .into_iter()
                    .map(|f| f.into_feature(ctx))
                    .collect::<Result<Vec<_>, Err>>()?,
            ))),
            AST::Name { inner, name } => Ok(Box::new(crate::features::Named::new(
                name,
                inner.into_feature(ctx)?,
            ))),
            AST::Assign(_, _) => unreachable!(),
            AST::Comment(_) => unreachable!(),
            AST::Cel(_) => unreachable!(),
            AST::VarRef(ident) => match ctx.definitions.get(&ident) {
                Some(var) => match var {
                    Variable::Geo(ast) => ast.clone().into_feature(ctx),
                    _ => Err(Err::BadType(ident)),
                },
                None => Err(Err::UndefinedVariable(ident)),
            },
        }
    }
}

fn parse_cel(i: &str) -> IResult<&str, AST, VerboseError<&str>> {
    let (start, _) = multispace0(i)?;
    let (i, exp) = context(
        "cel",
        delimited(tag("!{"), cut(take_while(|c| c != '}')), tag("}")),
    )(start)?;

    if let Err(_) = cel_interpreter::Program::compile(exp) {
        return Err(nom::Err::Error(VerboseError {
            errors: vec![(
                start,
                nom::error::VerboseErrorKind::Nom(nom::error::ErrorKind::Satisfy),
            )],
        }));
    }

    Ok((i, AST::Cel(exp.to_string())))
}

fn parse_ident(i: &str) -> IResult<&str, String, VerboseError<&str>> {
    let (i, _) = multispace0(i)?;
    let (i, s) = context(
        "ident",
        take_while(|c| {
            c == '_' || (c >= '0' && c <= '9') || (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')
        }),
    )(i)?;
    Ok((i, s.into()))
}

fn parse_uint(i: &str) -> IResult<&str, usize, VerboseError<&str>> {
    let (i, _) = multispace0(i)?;
    let (i, s) = context("uint", take_while(|c| c == '-' || (c >= '0' && c <= '9')))(i)?;
    Ok((
        i,
        s.parse().map_err(|_e| {
            nom::Err::Error(VerboseError {
                errors: vec![(
                    i,
                    nom::error::VerboseErrorKind::Nom(nom::error::ErrorKind::Digit),
                )],
            })
        })?,
    ))
}

fn parse_float(i: &str) -> IResult<&str, Value, VerboseError<&str>> {
    let (i, _) = multispace0(i)?;

    // Handle referencing variables
    if let Ok((i, _)) = tag::<_, _, VerboseError<&str>>("$")(i) {
        let (i, ident) = parse_ident(i)?;
        return Ok((i, Value::Ref(ident)));
    }

    // Handle CEL expressions
    if let Ok((i, ast)) = parse_cel(i) {
        match ast {
            AST::Cel(exp) => {
                return Ok((i, Value::Cel(exp)));
            }
            _ => unreachable!(),
        }
    }

    let (i, s) = context(
        "float",
        take_while(|c| c == '.' || c == '+' || c == '-' || (c >= '0' && c <= '9')),
    )(i)?;

    Ok((
        i,
        Value::Float(s.parse().map_err(|_e| {
            nom::Err::Error(VerboseError {
                errors: vec![(
                    i,
                    nom::error::VerboseErrorKind::Nom(nom::error::ErrorKind::Digit),
                )],
            })
        })?),
    ))
}

fn parse_coords(i: &str) -> IResult<&str, (Value, Value), VerboseError<&str>> {
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

fn parse_inner(i: &str) -> IResult<&str, InnerAST, VerboseError<&str>> {
    let (i, _) = multispace0(i)?;

    let (i, inner) = delimited(
        tuple((tag("("), multispace0)),
        alt((
            map(tuple((tag("h"), parse_float)), |(_, f)| {
                InnerAST::ScrewHole(f)
            }),
            map(tag("h"), |_| InnerAST::ScrewHole(Value::Float(3.1))),
            map(tag("smiley"), |_| InnerAST::Smiley),
            parse_inner_msp,
        )),
        tuple((multispace0, tag(")"))),
    )(i)?;

    Ok((i, inner))
}

fn parse_inner_msp(i: &str) -> IResult<&str, InnerAST, VerboseError<&str>> {
    let (i, _) = tag_no_case("msp")(i)?;
    match context("msp details", parse_details)(i) {
        Ok((i2, deets)) => {
            let size = if let Some((x, y)) = deets.size {
                Some((x, y))
            } else if deets.extra.len() == 2 {
                Some((deets.extra[0].clone(), deets.extra[1].clone()))
            } else if deets.extra.len() == 1 {
                Some((deets.extra[0].clone(), deets.extra[0].clone()))
            } else {
                None
            };

            Ok((i2, InnerAST::MechanicalSolderPoint(size)))
        }
        Err(_) => Ok((i, InnerAST::MechanicalSolderPoint(None))),
    }
}

enum DetailFragment {
    Coord(Value, Value),
    Size(Value, Value),
    Radius(Value),
    Rounding(Value),
    Extra(Value),
}

#[derive(Debug, Default, Clone)]
struct Details {
    coords: Option<(Value, Value)>,
    size: Option<(Value, Value)>,
    radius: Option<Value>,
    extra: Vec<Value>,
    inner: Option<InnerAST>,
    rounded: Option<Value>,
}

impl Details {
    fn parse_pos(i: &str) -> IResult<&str, DetailFragment, VerboseError<&str>> {
        let (i, _) = multispace0(i)?;
        let (i, _t) = tag("@")(i)?;
        let (i, c) = cut(parse_coords)(i)?;
        Ok((i, DetailFragment::Coord(c.0, c.1)))
    }
    fn parse_extra(i: &str) -> IResult<&str, DetailFragment, VerboseError<&str>> {
        let (i, f) = parse_float(i)?;
        Ok((i, DetailFragment::Extra(f)))
    }
    fn parse_size(i: &str) -> IResult<&str, DetailFragment, VerboseError<&str>> {
        let (i, _) = multispace0(i)?;
        let (i, (_, _, _, _, c)) = tuple((
            alt((tag_no_case("size"), tag_no_case("s"))),
            multispace0,
            tag("="),
            multispace0,
            cut(parse_coords),
        ))(i)?;
        Ok((i, DetailFragment::Size(c.0, c.1)))
    }
    fn parse_radius(i: &str) -> IResult<&str, DetailFragment, VerboseError<&str>> {
        let (i, _) = multispace0(i)?;
        let (i, (_, _, _, _, r)) = tuple((
            alt((tag_no_case("radius"), tag_no_case("r"))),
            multispace0,
            tag("="),
            multispace0,
            cut(parse_float),
        ))(i)?;
        Ok((i, DetailFragment::Radius(r)))
    }
    fn parse_rounding(i: &str) -> IResult<&str, DetailFragment, VerboseError<&str>> {
        let (i, _) = multispace0(i)?;
        let (i, (_, _, _, _, r)) = tuple((
            alt((tag_no_case("round"), tag_no_case("r"))),
            multispace0,
            tag("="),
            multispace0,
            cut(parse_float),
        ))(i)?;
        Ok((i, DetailFragment::Rounding(r)))
    }

    fn with_inner(mut self, inner: Option<InnerAST>) -> Self {
        self.inner = inner;
        self
    }
}

fn parse_details(i: &str) -> IResult<&str, Details, VerboseError<&str>> {
    let (i, _) = multispace0(i)?;

    let (i, deets) = delimited(
        tuple((tag("<"), multispace0)),
        cut(fold_many1(
            alt((
                tuple((
                    context("pos", Details::parse_pos),
                    multispace0,
                    opt(tag(",")),
                )),
                tuple((
                    context("size", Details::parse_size),
                    multispace0,
                    opt(tag(",")),
                )),
                tuple((
                    context("radius", Details::parse_radius),
                    multispace0,
                    opt(tag(",")),
                )),
                tuple((
                    context("rounding", Details::parse_rounding),
                    multispace0,
                    opt(tag(",")),
                )),
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
                    DetailFragment::Rounding(r) => {
                        acc.rounded = Some(r);
                    }
                    DetailFragment::Extra(f) => acc.extra.push(f),
                }
                acc
            },
        )),
        tuple((tag(">"), multispace0)),
    )(i)?;

    let (i, inner) = opt(parse_inner)(i)?;
    Ok((i, deets.with_inner(inner)))
}

fn parse_rect(i: &str) -> IResult<&str, AST, VerboseError<&str>> {
    let (i, _) = multispace0(i)?;
    let (i, _) = tag_no_case("R")(i)?;
    let (i, deets) = context("rectangle details", parse_details)(i)?;

    let size = if let Some((x, y)) = deets.size {
        Some((x, y))
    } else if deets.extra.len() == 2 {
        Some((deets.extra[0].clone(), deets.extra[1].clone()))
    } else if deets.extra.len() == 1 {
        Some((deets.extra[0].clone(), deets.extra[0].clone()))
    } else {
        None
    };

    Ok((
        i,
        AST::Rect {
            size,
            coords: deets.coords,
            inner: deets.inner,
            rounded: deets.rounded,
        },
    ))
}

fn parse_circle(i: &str) -> IResult<&str, AST, VerboseError<&str>> {
    let (i, _) = multispace0(i)?;
    let (i, _) = tag_no_case("C")(i)?;
    let (i2, deets) = context("circle details", parse_details)(i)?;

    let r = if let Some(r) = deets.radius {
        r
    } else if deets.extra.len() == 1 {
        deets.extra[0].clone()
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

fn parse_triangle(i: &str) -> IResult<&str, AST, VerboseError<&str>> {
    let (i, _) = multispace0(i)?;
    let (i, _) = tag_no_case("T")(i)?;
    let (i2, deets) = context("triangle details", cut(parse_details))(i)?;

    let size = if let Some((x, y)) = deets.size {
        (x, y)
    } else if deets.extra.len() == 2 {
        (deets.extra[0].clone(), deets.extra[1].clone())
    } else if deets.extra.len() == 1 {
        (deets.extra[0].clone(), deets.extra[0].clone())
    } else {
        return Err(nom::Err::Failure(nom::error::make_error(
            i,
            nom::error::ErrorKind::Satisfy,
        )));
    };

    Ok((
        i2,
        AST::Triangle {
            size,
            inner: deets.inner,
        },
    ))
}

fn parse_rmount(i: &str) -> IResult<&str, AST, VerboseError<&str>> {
    let (i, _) = multispace0(i)?;
    let (i, dir) = alt((
        tag_no_case("mount_cut_left"),
        tag_no_case("mount_cut_right"),
        tag_no_case("mount_cut_down"),
        tag_no_case("mount_cut"),
    ))(i)?;
    let (i, deets) = context("mount details", cut(parse_details))(i)?;

    let depth = if deets.extra.len() == 1 {
        deets.extra[0].clone()
    } else {
        return Err(nom::Err::Failure(nom::error::make_error(
            i,
            nom::error::ErrorKind::Satisfy,
        )));
    };

    Ok((
        i,
        AST::RMount {
            depth,
            dir: match dir.to_lowercase().as_str() {
                "mount_cut_left" => crate::Direction::Left,
                "mount_cut_right" => crate::Direction::Right,
                "mount_cut_down" => crate::Direction::Down,
                _ => crate::Direction::Up,
            },
        },
    ))
}

fn parse_array(i: &str) -> IResult<&str, AST, VerboseError<&str>> {
    let (i, _) = multispace0(i)?;

    let (i, params) = context(
        "array",
        delimited(
            tuple((tag("["), multispace0)),
            cut(tuple((
                parse_uint,
                opt(tuple((multispace0, tag(";"), multispace0, one_of("UDRL")))),
                opt(tuple((
                    multispace0,
                    tag(";"),
                    multispace0,
                    alt((tag_no_case("vscore"), tag_no_case("v-score"))),
                ))),
            ))),
            tuple((tag("]"), multispace0)),
        ),
    )(i)?;
    let (i, geo) = parse_geo(i)?;

    let (num, dir, vscore) = params;
    let dir = if let Some((_, _, _, s)) = dir {
        match s {
            'L' => crate::Direction::Left,
            'R' => crate::Direction::Right,
            'U' => crate::Direction::Up,
            'D' => crate::Direction::Down,
            _ => {
                return Err(nom::Err::Failure(nom::error::make_error(
                    i,
                    nom::error::ErrorKind::Satisfy,
                )));
            }
        }
    } else {
        crate::Direction::Right
    };

    Ok((
        i,
        AST::Array {
            dir,
            num,
            inner: Box::new(geo),
            vscore: vscore.is_some(),
        },
    ))
}

fn parse_column_layout(i: &str) -> IResult<&str, AST, VerboseError<&str>> {
    let (i, _) = multispace0(i)?;

    let (i, (dir, _, pos, _, _, inners)) = context(
        "column",
        delimited(
            tuple((tag_no_case("column"), multispace0)),
            tuple((
                alt((
                    tag_no_case("left"),
                    tag_no_case("center"),
                    tag_no_case("right"),
                )),
                multispace0,
                opt(tuple((tag("@"), parse_coords))),
                multispace0,
                tag("{"),
                fold_many1(
                    tuple((parse_geo, multispace0, opt(tag(",")))),
                    Vec::new(),
                    |mut acc, (inner, _, _)| {
                        acc.push(Box::new(inner));
                        acc
                    },
                ),
            )),
            tuple((tag("}"), multispace0)),
        ),
    )(i)?;

    Ok((
        i,
        AST::ColumnLayout {
            align: match dir.to_lowercase().as_str() {
                "left" => crate::Align::Start,
                "right" => crate::Align::End,
                _ => crate::Align::Center,
            },
            inners: inners,
            coords: pos.map(|x| x.1),
        },
    ))
}

fn parse_pos_spec(i: &str) -> IResult<&str, WrapPosition, VerboseError<&str>> {
    let (i, (_, side, offset, _, align, _)) = tuple((
        multispace0,
        alt((
            tag_no_case("left"),
            tag_no_case("right"),
            tag_no_case("up"),
            tag_no_case("down"),
            tag_no_case("top"),
            tag_no_case("bottom"),
        )),
        opt(parse_float),
        multispace0,
        opt(tuple((
            multispace0,
            tag_no_case("align"),
            multispace0,
            alt((
                tag_no_case("center"),
                tag_no_case("exterior"),
                tag_no_case("interior"),
            )),
            multispace0,
        ))),
        tag("=>"),
    ))(i)?;

    Ok((
        i,
        WrapPosition::Cardinal {
            side: match side.to_lowercase().as_str() {
                "left" => Direction::Left,
                "right" => Direction::Right,
                "top" | "up" => Direction::Up,
                "bottom" | "down" => Direction::Down,
                _ => unreachable!(),
            },
            offset: offset.unwrap_or(Value::Float(0.0)),
            align: match align {
                Some((_, _, _, align, _)) => match align.to_lowercase().as_str() {
                    "exterior" => crate::Align::End,
                    "interior" => crate::Align::Start,
                    _ => crate::Align::Center,
                },
                _ => crate::Align::Center,
            },
        },
    ))
}

fn parse_about_spec(i: &str) -> IResult<&str, WrapPosition, VerboseError<&str>> {
    let (i, (_, angle, _, offset, _, _)) = tuple((
        tuple((multispace0, tag_no_case("angle("))),
        parse_float,
        tuple((multispace0, tag(")"))),
        opt(parse_float),
        multispace0,
        tag("=>"),
    ))(i)?;

    Ok((
        i,
        WrapPosition::Angle {
            angle,
            offset: offset.unwrap_or(Value::Float(0.0)),
        },
    ))
}

fn parse_wrap_center_spec(i: &str) -> IResult<&str, WrapPosition, VerboseError<&str>> {
    let (i, _) = tuple((
        tuple((multispace0, tag_no_case("center"))),
        multispace0,
        tag("=>"),
    ))(i)?;

    Ok((
        i,
        WrapPosition::Angle {
            angle: Value::Float(0.0),
            offset: Value::Float(0.0),
        },
    ))
}

fn parse_corner_spec(i: &str) -> IResult<&str, WrapPosition, VerboseError<&str>> {
    let (i, (_, opp, side, _, align, _)) = tuple((
        multispace0,
        alt((tag_no_case("min-"), tag_no_case("max-"))),
        alt((
            tag_no_case("left"),
            tag_no_case("right"),
            tag_no_case("up"),
            tag_no_case("down"),
            tag_no_case("top"),
            tag_no_case("bottom"),
        )),
        multispace0,
        opt(tuple((
            multispace0,
            tag_no_case("align"),
            multispace0,
            alt((
                tag_no_case("center"),
                tag_no_case("exterior"),
                tag_no_case("interior"),
            )),
            multispace0,
        ))),
        tag("=>"),
    ))(i)?;

    Ok((
        i,
        WrapPosition::Corner {
            side: match side.to_lowercase().as_str() {
                "left" => Direction::Left,
                "right" => Direction::Right,
                "top" | "up" => Direction::Up,
                "bottom" | "down" => Direction::Down,
                _ => unreachable!(),
            },
            opposite: match opp.to_lowercase().as_str() {
                "min-" => false,
                "max-" => true,
                _ => unreachable!(),
            },
            align: match align {
                Some((_, _, _, align, _)) => match align.to_lowercase().as_str() {
                    "exterior" => crate::Align::End,
                    "interior" => crate::Align::Start,
                    _ => crate::Align::Center,
                },
                _ => crate::Align::Center,
            },
        },
    ))
}

fn parse_wrap(i: &str) -> IResult<&str, AST, VerboseError<&str>> {
    let (i, _) = multispace0(i)?;

    let (i, (_, _, _, inner, _, _, _, _, _, _)) = context(
        "wrap",
        tuple((
            tag_no_case("wrap"),
            multispace0,
            tag("("),
            parse_geo,
            multispace0,
            tag(")"),
            multispace0,
            tag_no_case("with"),
            multispace0,
            tag("{"),
        )),
    )(i)?;
    let (i, elements) = fold_many1(
        context(
            "pos spec",
            alt((
                nom::combinator::map(
                    tuple((
                        alt((
                            parse_pos_spec,
                            parse_about_spec,
                            parse_wrap_center_spec,
                            parse_corner_spec,
                        )),
                        multispace0,
                        parse_geo,
                        multispace0,
                        opt(tag(",")),
                    )),
                    |s| Some(s),
                ),
                nom::combinator::map(parse_comment, |_| None),
            )),
        ),
        Vec::new(),
        |mut acc, feature| {
            if let Some((pos, _, feature, _, _)) = feature {
                acc.push((pos, Box::new(feature)));
            }
            acc
        },
    )(i)?;
    let (i, _) = tuple((multispace0, tag("}"), multispace0))(i)?;

    Ok((
        i,
        AST::Wrap {
            inner: Box::new(inner),
            features: elements,
        },
    ))
}

fn parse_assign(i: &str) -> IResult<&str, AST, VerboseError<&str>> {
    let (i, _) = multispace0(i)?;

    let (i, (_, _, var, _, _, geo, _, _)) = tuple((
        tag("let"),
        multispace0,
        parse_ident,
        multispace0,
        tag("="),
        parse_geo,
        multispace0,
        opt(tag(";")),
    ))(i)?;

    Ok((i, AST::Assign(var, Box::new(geo))))
}

fn parse_var(i: &str) -> IResult<&str, AST, VerboseError<&str>> {
    let (i, _) = multispace0(i)?;
    let (i, (_, var)) = tuple((tag("$"), parse_ident))(i)?;
    Ok((i, AST::VarRef(var)))
}

pub fn parse_comment(i: &str) -> IResult<&str, AST, VerboseError<&str>> {
    let (i, _) = multispace0(i)?;
    let (i, _) = alt((tag("#"), tag("//")))(i)?;
    let (i, v) = take_while(|chr| chr != '\n')(i)?;
    Ok((i, AST::Comment(v.to_string())))
}

fn parse_tuple(i: &str) -> IResult<&str, AST, VerboseError<&str>> {
    let (i, _) = multispace0(i)?;
    let (i, _) = tag("(")(i)?;

    let (i, elements) = fold_many1(
        tuple((multispace0, parse_geo, multispace0, opt(tag(",")))),
        Vec::new(),
        |mut acc, (_, feature, _, _)| {
            acc.push(Box::new(feature));
            acc
        },
    )(i)?;
    let (i, _) = tuple((multispace0, tag(")"), multispace0))(i)?;

    Ok((i, AST::Tuple { inners: elements }))
}

fn parse_negative(i: &str) -> IResult<&str, AST, VerboseError<&str>> {
    let (i, _) = multispace0(i)?;

    let (i, (_, _, inners)) = context(
        "negative",
        delimited(
            tuple((tag_no_case("negative"), multispace0)),
            tuple((
                multispace0,
                tag("{"),
                fold_many1(
                    tuple((parse_geo, multispace0, opt(tag(",")))),
                    Vec::new(),
                    |mut acc, (inner, _, _)| {
                        acc.push(Box::new(inner));
                        acc
                    },
                ),
            )),
            tuple((tag("}"), multispace0)),
        ),
    )(i)?;

    Ok((i, AST::Negative { inners: inners }))
}

fn parse_rotate(i: &str) -> IResult<&str, AST, VerboseError<&str>> {
    let (i, _) = multispace0(i)?;

    let (i, (_, _, _, rotation, _, _, _)) = context(
        "rotate",
        tuple((
            tag_no_case("rotate"),
            multispace0,
            tag("("),
            parse_float,
            multispace0,
            tag(")"),
            multispace0,
        )),
    )(i)?;

    let (i, (_, inners)) = context(
        "rotate_body",
        delimited(
            tag("{"),
            tuple((
                multispace0,
                fold_many1(
                    tuple((parse_geo, multispace0, opt(tag(",")))),
                    Vec::new(),
                    |mut acc, (inner, _, _)| {
                        acc.push(Box::new(inner));
                        acc
                    },
                ),
            )),
            tuple((tag("}"), multispace0)),
        ),
    )(i)?;

    Ok((
        i,
        AST::Rotate {
            rotation: rotation,
            inners: inners,
        },
    ))
}

fn parse_geo(i: &str) -> IResult<&str, AST, VerboseError<&str>> {
    let (i, feature) = alt((
        parse_assign,
        parse_cel,
        parse_array,
        parse_rect,
        parse_circle,
        parse_triangle,
        parse_rmount,
        parse_wrap,
        parse_column_layout,
        parse_var,
        parse_tuple,
        parse_negative,
        parse_rotate,
        parse_comment,
    ))(i)?;

    let (i, name) = opt(tuple((multispace0, tag("%"), parse_ident)))(i)?;

    if let Some((_, _, name)) = name {
        return Ok((
            i,
            AST::Name {
                name: name,
                inner: Box::new(feature),
            },
        ));
    }

    Ok((i, feature))
}

/// Parses the provided panel spec and returns the series of features
/// it represents.
pub fn build<'a>(i: &str) -> Result<Vec<Box<dyn super::Feature + 'a>>, Err> {
    let mut ctx = ResolverContext::default();
    let (_, (g, _)) = all_consuming(tuple((many0(parse_geo), multispace0)))(i).map_err(|e| {
        Err::Parse(nom::error::convert_error(
            i,
            match e {
                nom::Err::Error(e) | nom::Err::Failure(e) => e,
                _ => unreachable!(),
            },
        ))
    })?;

    g.into_iter()
        .map(|g| match g {
            AST::Assign(var, geo) => {
                ctx.handle_assignment(var, geo);
                None
            }
            AST::Comment(_) => None,
            _ => Some(g.into_feature(&mut ctx)),
        })
        .filter(|f| f.is_some())
        .map(|f| f.unwrap())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect() {
        let out = parse_geo("R<@(1,2)>");
        assert!(
            matches!(out, Ok(("", AST::Rect{ coords: Some((Value::Float(x), Value::Float(y))), size: None, inner: _, rounded: None })) if
                x > 0.99 && x < 1.01 && y > 1.99 && y < 2.01
            )
        );

        let out = parse_geo("R<@(1,2), 2, 4>");
        assert!(
            matches!(out, Ok(("", AST::Rect{ coords: Some((Value::Float(x), Value::Float(y))), size: Some((Value::Float(w), Value::Float(h))), inner: _, rounded: None })) if
                x > 0.99 && x < 1.01 && y > 1.99 && y < 2.01 &&
                w > 1.99 && w < 2.01 && h > 3.99 && h < 4.01
            )
        );

        let out = parse_geo("R<@(1,2), 4>");
        assert!(
            matches!(out, Ok(("", AST::Rect{ coords: Some((Value::Float(x), Value::Float(y))), size: Some((Value::Float(w), Value::Float(h))), inner: _, rounded: None })) if
                x > 0.99 && x < 1.01 && y > 1.99 && y < 2.01 &&
                w > 3.99 && w < 4.01 && h > 3.99 && h < 4.01
            )
        );

        let out = parse_geo("R<4>");
        assert!(
            matches!(out, Ok(("", AST::Rect{ coords: None, size: Some((Value::Float(w), Value::Float(h))), inner: _, rounded: None })) if
                w > 3.99 && w < 4.01 && h > 3.99 && h < 4.01
            )
        );

        let out = parse_geo("R<@(1,2), size = (2,4)>");
        assert!(
            matches!(out, Ok(("", AST::Rect{ coords: Some((Value::Float(x), Value::Float(y))), size: Some((Value::Float(w), Value::Float(h))), inner: _, rounded: None })) if
                x > 0.99 && x < 1.01 && y > 1.99 && y < 2.01 &&
                w > 1.99 && w < 2.01 && h > 3.99 && h < 4.01
            )
        );

        let out = parse_geo(" R<6>(h)");
        assert!(
            matches!(out, Ok(("", AST::Rect{ coords: None, size: Some((Value::Float(w), Value::Float(h))), inner: Some(InnerAST::ScrewHole(Value::Float(dia))), rounded: None })) if
                w > 5.99 && w < 6.01 && h > 5.99 && h < 6.01 &&
                dia < 3.11 && dia > 3.09
            )
        );

        let out = parse_geo(" R<6, round = 2>(h)");
        assert!(
            matches!(out, Ok(("", AST::Rect{ coords: None, size: Some((Value::Float(w), Value::Float(h))), inner: Some(InnerAST::ScrewHole(Value::Float(dia))), rounded: Some(_) })) if
                w > 5.99 && w < 6.01 && h > 5.99 && h < 6.01 &&
                dia < 3.11 && dia > 3.09
            )
        );
    }

    #[test]
    fn test_circle() {
        let out = parse_geo("C < @ ( 2 , 1 ), 4.5>");
        assert!(
            matches!(out, Ok(("", AST::Circle{ coords: Some((Value::Float(x), Value::Float(y))), radius: r, inner: _ })) if
                y > 0.99 && y < 1.01 && x > 1.99 && x < 2.01 &&
                r.float() > 4.49 && r.float() < 4.51
            )
        );

        let out = parse_geo("C<@(2, 1), 3.5>");
        assert!(
            matches!(out, Ok(("", AST::Circle{ coords: Some((Value::Float(x), Value::Float(y))), radius: r, inner: _ })) if
                y > 0.99 && y < 1.01 && x > 1.99 && x < 2.01 &&
                r.float() > 3.49 && r.float() < 3.51
            )
        );
        let out = parse_geo("C<3.5>");
        assert!(
            matches!(out, Ok(("", AST::Circle{ coords: None, radius: r, inner: _ })) if
                r.float() > 3.49 && r.float() < 3.51
            )
        );

        let out = parse_geo("C<@(2, 1), R=3.5>");
        assert!(
            matches!(out, Ok(("", AST::Circle{ coords: Some((Value::Float(x), Value::Float(y))), radius: r, inner: _ })) if
                y > 0.99 && y < 1.01 && x> 1.99 && x < 2.01 &&
                r.float() > 3.49 && r.float() < 3.51
            )
        );

        let out = parse_geo("C<3.5> ( h9 )");
        assert!(
            matches!(out, Ok(("", AST::Circle{ coords: None, radius: r, inner: Some(InnerAST::ScrewHole(Value::Float(dia))) })) if
                r.float() > 3.49 && r.float() < 3.51 && dia > 8.999 && dia < 9.001
            )
        );
    }

    #[test]
    fn test_msp() {
        let out = parse_geo("C<5>(msp)");
        assert!(matches!(
            out,
            Ok((
                "",
                AST::Circle {
                    inner: Some(InnerAST::MechanicalSolderPoint(None)),
                    ..
                },
            ))
        ));

        let out = parse_geo("C<5>(msp<2,1>)");
        assert!(matches!(
            out,
            Ok((
                "",
                AST::Circle { inner: Some(InnerAST::MechanicalSolderPoint(Some((Value::Float(w), Value::Float(h))))), .. },
            )) if w > 1.99 && w < 2.01 && h > 0.99 && h < 1.01
        ));
    }

    #[test]
    fn test_triangle() {
        let out = parse_geo("T<2,1>");
        assert!(
            matches!(out, Ok(("", AST::Triangle{ size: (Value::Float(x), Value::Float(y)), inner: _ })) if
                y > 0.99 && y < 1.01 && x > 1.99 && x < 2.01
            )
        );
    }

    #[test]
    fn test_r_mount() {
        let out = parse_geo("mount_cut<12>");
        assert!(matches!(out, Ok(("", AST::RMount{ depth, dir })) if
            depth.float() > 11.99 && depth.float() < 12.01 && dir == crate::Direction::Up
        ));
    }

    #[test]
    fn test_array() {
        let out = parse_geo("[5]C<4.5>");
        assert!(
            matches!(out, Ok(("", AST::Array{ num: 5, inner: b, dir: crate::Direction::Right, vscore: false})) if
                matches!(&*b, AST::Circle{ radius, .. } if radius.float() > 4.4 && radius.float() < 4.6)
            )
        );

        let out = parse_geo("[5; D; v-score]C<4.5>");
        assert!(
            matches!(out, Ok(("", AST::Array{ num: 5, inner: b, dir: crate::Direction::Down, vscore: true})) if
                matches!(&*b, AST::Circle{ radius, .. } if radius.float() > 4.4 && radius.float() < 4.6)
            )
        );
    }

    #[test]
    fn test_column_layout() {
        let out = parse_geo("column left { R<5> }");
        assert!(matches!(
            out,
            Ok((
                "",
                AST::ColumnLayout {
                    align: crate::Align::Start,
                    inners: i,
                    coords: None,
                },
            ))
            if i.len() == 1
        ));

        let out = parse_geo("column RiGHt { C<5> R<1> }");
        assert!(matches!(
            out,
            Ok((
                "",
                AST::ColumnLayout {
                    align: crate::Align::End,
                    inners: i,
                    coords: None,
                },
            ))
            if i.len() == 2
        ));

        let out = parse_geo("column center { R<1> }");
        // eprintln!("{:?}", out);
        assert!(matches!(
            out,
            Ok((
                "",
                AST::ColumnLayout {
                    align: crate::Align::Center,
                    inners: i,
                    coords: None,
                },
            ))
            if i.len() == 1
        ));

        let out = parse_geo("column center { (R<1>) }");
        // eprintln!("{:?}", out);
        assert!(matches!(
            out,
            Ok((
                "",
                AST::ColumnLayout {
                    align: crate::Align::Center,
                    inners: i,
                    coords: None,
                },
            ))
            if i.len() == 1 && matches!(*i[0], AST::Tuple{ .. })
        ));

        let out = parse_geo("column center @(1, 2) { R<1> }");
        eprintln!("{:?}", out);
        assert!(matches!(
            out,
            Ok((
                "",
                AST::ColumnLayout {
                    align: crate::Align::Center,
                    inners: i,
                    coords: Some((Value::Float(x), Value::Float(y))),
                },
            ))
            if i.len() == 1 && x > 0.99 && x < 1.01 && y > 1.99 && y < 2.01
        ));
    }

    #[test]
    fn test_wrap() {
        let out = parse_geo(
            "wrap ($inner) with { left-0.5 => C<2>(h), right align exterior => C<2>(h4) }",
        );
        // eprintln!("{:?}", out);
        assert!(matches!(out, Ok(("", AST::Wrap { inner, features })) if
            matches!(*inner, AST::VarRef(ref var) if var == "inner") && features.len() == 2 &&
            matches!(features[1].0, WrapPosition::Cardinal{ align: crate::Align::End, .. })
        ));

        let out = parse_geo(
            "wrap(column center {[12] R<5>(h)}) with {left-0.5 => C<2>(h), right+0.5 => C<2>(h)}",
        );
        // eprintln!("{:?}", out);
        assert!(matches!(out, Ok(("", AST::Wrap { inner, features })) if
            matches!(*inner, AST::ColumnLayout{ .. }) && features.len() == 2 &&
            matches!(features[0].0, WrapPosition::Cardinal{ side: Direction::Left, offset: Value::Float(o1), .. } if
            o1 < -0.4 && o1 > -0.6) &&
            matches!(features[1].0, WrapPosition::Cardinal{ side: Direction::Right, offset: Value::Float(o2), .. } if
           o2 > 0.4 && o2 < 0.6)
        ));

        let out = parse_geo(
            "wrap ($inner) with {\n  left => (C<2>(h), C<2>),\n # test comment\n right => (C<2>(h), C<2>),\n}",
        );
        // eprintln!("{:?}", out);
        assert!(matches!(out, Ok(("", AST::Wrap { inner, features })) if
            matches!(*inner, AST::VarRef(ref var) if var == "inner") && features.len() == 2 &&
            matches!(features[0].0, WrapPosition::Cardinal{ align: crate::Align::Center, .. })
        ));

        let out =
            parse_geo("wrap ($inner) with {\n  angle(90) => C<2>,\n  angle(-90) 25 => C<2>,\n}");
        // eprintln!("{:?}", out);
        assert!(matches!(out, Ok(("", AST::Wrap { inner, features })) if
            matches!(*inner, AST::VarRef(ref var) if var == "inner") && features.len() == 2 &&
            matches!(features[0].0, WrapPosition::Angle{ angle: Value::Float(a1), .. } if
            a1 < 91. && a1 > 89.) &&
            matches!(features[1].0, WrapPosition::Angle{ offset: Value::Float(o2), .. } if
           o2 > 24. && o2 < 26.)
        ));

        let out = parse_geo("wrap ($inner) with {\n  center => C<2>,\n}");
        assert!(matches!(out, Ok(("", AST::Wrap { inner, features })) if
            matches!(*inner, AST::VarRef(ref var) if var == "inner") && features.len() == 1 &&
            matches!(features[0].0, WrapPosition::Angle{ angle: Value::Float(a1), .. } if
            a1 < 0.1 && a1 > -0.1)
        ));

        let out = parse_geo("wrap ($inner) with {\n  min-left align exterior => C<2>,\n}");
        // eprintln!("{:?}", out);
        assert!(matches!(out, Ok(("", AST::Wrap { inner, features })) if
            matches!(*inner, AST::VarRef(ref var) if var == "inner") && features.len() == 1 &&
            matches!(features[0].0, WrapPosition::Corner{ side: Direction::Left, align: crate::Align::End, opposite: false})
        ));
    }

    #[test]
    fn test_tuple() {
        let out = parse_geo("(C<2>(h))");
        eprintln!("{:?}", out);
        assert!(
            matches!(out, Ok(("", AST::Tuple{ inners })) if inners.len() == 1 &&
                matches!(&*inners[0], AST::Circle{ coords: None, radius: r, inner: Some(_) }  if
                    r.float() > 1.99 && r.float() < 2.01
                )
            )
        );

        let out = parse_geo("(C<2>(h), R<4>)");
        assert!(
            matches!(out, Ok(("", AST::Tuple{ inners })) if inners.len() == 2 &&
                matches!(*inners[1], AST::Rect{ coords: None, size: Some((Value::Float(w), Value::Float(h))), inner: _, rounded: None } if
                    w > 3.99 && w < 4.01 && h > 3.99 && h < 4.01
                )
            )
        );

        let out = parse_geo("(C<2>(h), (C<2>(h), R<4>))");
        // eprintln!("{:?}", out);
        assert!(
            matches!(out, Ok(("", AST::Tuple{ inners })) if inners.len() == 2 &&
                matches!(&*inners[1], AST::Tuple{ inners } if inners.len() == 2 &&
                    matches!(*inners[1], AST::Rect{ coords: None, size: Some((Value::Float(w), Value::Float(h))), inner: _, rounded: None } if
                        w > 3.99 && w < 4.01 && h > 3.99 && h < 4.01
                    )
                )
            )
        );
    }

    #[test]
    fn test_negative() {
        let out = parse_geo("negative{C<2>}");
        //eprintln!("{:?}", out);
        assert!(
            matches!(out, Ok(("", AST::Negative{ inners })) if inners.len() == 1 &&
                matches!(&*inners[0], AST::Circle{ coords: None, radius: r, inner: None }  if
                    r.float() > 1.99 && r.float() < 2.01
                )
            )
        );

        let out = parse_geo("negative {\n C<2>,\n   R<4>\n\n}");
        // eprintln!("{:?}", out);
        assert!(
            matches!(out, Ok(("", AST::Negative{ inners })) if inners.len() == 2 &&
                matches!(&*inners[0], AST::Circle{ coords: None, radius: r, inner: None }  if
                    r.float() > 1.99 && r.float() < 2.01
                ) &&
                matches!(*inners[1], AST::Rect{ coords: None, size: Some((Value::Float(w), Value::Float(h))), inner: _, rounded: None }  if
                    w > 3.99 && w < 4.01 && h > 3.99 && h < 4.01
                )
            )
        );
    }

    #[test]
    fn test_var() {
        let out = parse_geo("let bleh = C<25>");
        // eprintln!("{:?}", out);
        assert!(matches!(out, Ok(("", AST::Assign(var, circ))) if
            var == "bleh".to_string() && matches!(*circ, AST::Circle{ .. })));

        let out = parse_geo("$bleh");
        // eprintln!("{:?}", out);
        assert!(matches!(out, Ok(("", AST::VarRef(var))) if var == "bleh".to_string()));

        let out = build(
            "let rect = column center {
          [12] R<7.5>(h)
          [11] R<7.5>(h)
          [12] R<7.5>(h)
        }$rect",
        );
        // eprintln!("{:?}", out);
        assert!(matches!(out, Ok(features) if features.len() == 1));
    }

    #[test]
    fn test_err_msgs() {
        let out = build("C<a>");
        assert!(matches!(out, Err(Err::Parse(_))));
        let out = build("T<a>");
        assert!(matches!(out, Err(Err::Parse(_))));

        let out = build("R<@(a)>");
        // eprintln!("\n\n{}\n\n", match out.err().unwrap() {
        //     Err::Parse(e) => e,
        //     _ => unreachable!(),
        // });
        // unreachable!();
        assert!(matches!(out, Err(Err::Parse(_))));

        let out = build("(aBC)");
        assert!(matches!(out, Err(Err::Parse(_))));

        let out = build("let bleh = !{aa$%dsfsd + 44}");
        assert!(matches!(out, Err(Err::Parse(_))));
    }

    #[test]
    fn test_cel() {
        let out = parse_geo("let bleh = !{44}");
        assert!(matches!(out, Ok(("", AST::Assign(var, exp))) if
            var == "bleh".to_string() && matches!(*exp, AST::Cel(_))));

        let out = build("let bleh = !{1 + 1}");
        assert!(matches!(out, Ok(_)));
        let out = build("let bleh = !{1 + 1}\nlet ye = !{bleh + 2}");
        assert!(matches!(out, Ok(_)));

        let out = build("let v2 = !{1 * 1}\nR<$v2>");
        assert!(matches!(out, Ok(v) if v.len() == 1));

        let out = build("R<$missing>");
        // eprintln!("{:?}", out);
        assert!(matches!(out, Err(Err::UndefinedVariable(_))));

        let out = build("let bleh = !{22};\nR<!{bleh + 1}>");
        assert!(matches!(out, Ok(v) if v.len() == 1));

        let out = build("let bleh = !{5};\nwrap (R<5>) with { left $bleh => R<2>, }");
        // eprintln!("{:?}", out);
        assert!(matches!(out, Ok(v) if v.len() == 1));
    }

    #[test]
    fn test_comment() {
        let out = parse_geo("# yooooooo");
        assert!(matches!(out, Ok(("", AST::Comment(msg))) if
            msg == " yooooooo"
        ));

        let out = parse_geo("// yeeeeeeeee");
        assert!(matches!(out, Ok(("", AST::Comment(msg))) if
            msg == " yeeeeeeeee"
        ));
    }

    #[test]
    fn test_rotate() {
        let out = parse_geo("rotate(45.0){C<2>}");
        eprintln!("{:?}", out);
        assert!(
            matches!(out, Ok(("", AST::Rotate{ rotation, inners })) if inners.len() == 1 &&
                matches!(&*inners[0], AST::Circle{ coords: None, radius: r, inner: None }  if
                    r.float() > 1.99 && r.float() < 2.01
                ) &&
                matches!(rotation.float(), f if f < 45.01 && f > 44.99)
            )
        );
    }

    #[test]
    fn test_name() {
        let out = parse_geo("C<2> % circle1");
        eprintln!("{:?}", out);
        assert!(matches!(out, Ok(("", AST::Name{ name, inner })) if
            matches!(&*inner, AST::Circle{ coords: None, radius: r, inner: None }  if
                r.float() > 1.99 && r.float() < 2.01
            ) &&
            name == "circle1"
        ));
    }
}
