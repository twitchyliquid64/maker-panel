use maker_panel::{
    features::{repeating, AtPos, Circle, Column, Rect, ScrewHole},
    Direction, Err, Layer, Panel,
};
use std::path::PathBuf;
use structopt::StructOpt;

/// Represents an output format provided for the gen command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fmt {
    Edge,
    FrontCopper,
    FrontMask,
    FrontLegend,
    BackCopper,
    BackMask,
    BackLegend,
}

impl Fmt {
    fn file_suffix(&self) -> &'static str {
        match self {
            Fmt::Edge => "Edge.Cuts.gm1",
            Fmt::FrontCopper => "F.Cu.gtl",
            Fmt::FrontMask => "F.Mask.gts",
            Fmt::FrontLegend => "F.SilkS.gto",
            Fmt::BackCopper => "B.Cu.gbl",
            Fmt::BackMask => "B.Mask.gbs",
            Fmt::BackLegend => "B.SilkS.gto",
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
            "f.legend" => Ok(Fmt::FrontLegend),
            "b.cu" => Ok(Fmt::BackCopper),
            "b.mask" => Ok(Fmt::BackMask),
            "b.legend" => Ok(Fmt::BackLegend),
            _ => Err(format!("no such fmt: {}", s).to_string()),
        }
    }
}

/// Represents the --size parameter from the command line.
#[derive(Debug, PartialEq, Clone)]
pub struct RenderFitTo(usvg::FitTo);

impl std::str::FromStr for RenderFitTo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "mm" {
            return Ok(RenderFitTo(usvg::FitTo::Original));
        };

        if s.starts_with("z:") {
            return Ok(RenderFitTo(usvg::FitTo::Zoom(
                s[2..]
                    .parse::<f32>()
                    .map_err(|e| format!("invalid zoom: {}", e))?,
            )));
        };

        Ok(RenderFitTo(usvg::FitTo::Width(
            s.parse::<u32>()
                .map_err(|e| format!("invalid width: {}", e))?,
        )))
    }
}

#[derive(StructOpt, Debug, PartialEq, Clone)]
pub enum Cmd {
    #[structopt(name = "png", about = "Renders a PNG visualizing the panel.")]
    Render {
        #[structopt(
            name = "size",
            short = "s",
            long = "size",
            about = "Specify z:<zoom> or width in pixels",
            default_value = "z:21.0"
        )]
        fit_to: RenderFitTo,

        output: PathBuf,
    },
    #[structopt(name = "gen", about = "Generates CAD files.")]
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

    run_cmd(args, panel);
}

fn run_cmd(args: Opt, panel: Panel) {
    let mut stdout = std::io::stdout();

    match args.cmd {
        Cmd::Render { output, fit_to } => {
            let n = panel.make_svg().unwrap();
            // println!("{}", n.to_string(usvg::XmlOptions::default()));
            resvg::render_node(&n.root(), fit_to.0, Some(usvg::Color::white()))
                .unwrap()
                .save_png(output)
                .unwrap();
        }
        Cmd::Gen { fmt } => match fmt {
            Fmt::Edge => panel.serialize_gerber_edges(&mut stdout).unwrap(),
            Fmt::FrontCopper => panel
                .serialize_gerber_layer(Layer::FrontCopper, &mut stdout)
                .unwrap(),
            Fmt::FrontMask => panel
                .serialize_gerber_layer(Layer::FrontMask, &mut stdout)
                .unwrap(),
            Fmt::FrontLegend => panel
                .serialize_gerber_layer(Layer::FrontLegend, &mut stdout)
                .unwrap(),
            Fmt::BackCopper => panel
                .serialize_gerber_layer(Layer::BackCopper, &mut stdout)
                .unwrap(),
            Fmt::BackMask => panel
                .serialize_gerber_layer(Layer::BackMask, &mut stdout)
                .unwrap(),
            Fmt::BackLegend => panel
                .serialize_gerber_layer(Layer::BackLegend, &mut stdout)
                .unwrap(),
        },
    };
}
