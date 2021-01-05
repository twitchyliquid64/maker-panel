use maker_panel::{
    features::{repeating, AtPos, Circle, Column, Rect, ScrewHole},
    Direction, Err, Layer, Panel,
};
use std::path::PathBuf;
use structopt::StructOpt;

const DEFAULT_FIT: usvg::FitTo = usvg::FitTo::Zoom(23.);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fmt {
    Edge,
    FrontCopper,
    FrontMask,
    BackCopper,
    BackMask,
}

impl Fmt {
    fn file_suffix(&self) -> &'static str {
        match self {
            Fmt::Edge => "Edge.Cuts.gm1",
            Fmt::FrontCopper => "F.Cu.gtl",
            Fmt::FrontMask => "F.Mask.gts",
            Fmt::BackCopper => "B.Cu.gbl",
            Fmt::BackMask => "B.Mask.gbs",
        }
    }
}

impl std::str::FromStr for Fmt {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "edge" => Ok(Fmt::Edge),
            "f.cu" => Ok(Fmt::FrontCopper),
            "f.mask" => Ok(Fmt::FrontMask),
            "b.cu" => Ok(Fmt::BackCopper),
            "b.mask" => Ok(Fmt::BackMask),
            _ => Err(format!("no such fmt: {}", s).to_string()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(StructOpt, Debug, PartialEq, Clone)]
pub enum Cmd {
    Gen { fmt: Fmt },
}

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    name = "maker-panel",
    about = "Generates mechanical PCBs based on repeating geometry"
)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Cmd,
}

fn main() {
    let args = Opt::from_args();

    let mut panel = Panel::new();
    // panel.convex_hull(true);
    // panel.push(Rect::with_center([0.0, -2.5].into(), 5., 5.));
    panel.push(AtPos::x_ends(
        Column::align_center(vec![
            repeating::Tile::new(
                Rect::with_inner(ScrewHole::default(), [0., 0.].into(), [5., 5.].into()),
                Direction::Right,
                8,
            ),
            repeating::Tile::new(
                Rect::with_inner(ScrewHole::default(), [0., 0.].into(), [5., 5.].into()),
                Direction::Right,
                5,
            ),
            repeating::Tile::new(
                Rect::with_inner(ScrewHole::default(), [0., 0.].into(), [5., 5.].into()),
                Direction::Right,
                8,
            ),
        ]),
        Some(Circle::wrap_with_radius(ScrewHole::with_diameter(5.), 7.5)),
        Some(Circle::wrap_with_radius(ScrewHole::with_diameter(5.), 7.5)),
    ));
    // panel.push(Circle::new([0., 7.5].into(), 7.5));
    // panel.push(Circle::new([20., 7.5].into(), 7.5));

    let n = panel.make_svg().unwrap();

    // println!("{}", n.to_string(usvg::XmlOptions::default()));
    resvg::render_node(&n.root(), DEFAULT_FIT, Some(usvg::Color::white()))
        .unwrap()
        .save_png("/tmp/ye.png")
        .unwrap();

    let mut stdout = std::io::stdout();

    match args.cmd {
        Cmd::Gen { fmt } => match fmt {
            Fmt::Edge => panel.serialize_gerber_edges(&mut stdout).unwrap(),
            Fmt::FrontCopper => panel
                .serialize_gerber_layer(Layer::FrontCopper, &mut stdout)
                .unwrap(),
            Fmt::FrontMask => panel
                .serialize_gerber_layer(Layer::FrontMask, &mut stdout)
                .unwrap(),
            Fmt::BackCopper => panel
                .serialize_gerber_layer(Layer::BackCopper, &mut stdout)
                .unwrap(),
            Fmt::BackMask => panel
                .serialize_gerber_layer(Layer::BackMask, &mut stdout)
                .unwrap(),
        },
    };
}
