use maker_panel::{
    features::{repeating, AtPos, Circle, Column, Rect, ScrewHole},
    Direction, Layer, Panel,
};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug)]
enum Err {
    IO(std::io::Error),
    General(maker_panel::Err),
    Zip(zip::result::ZipError),
}

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
    PlatedDrill,
    NonPlatedDrill,
    Zip,
}

impl Fmt {
    fn all_formats() -> &'static [Fmt] {
        &[
            Fmt::Edge,
            Fmt::FrontCopper,
            Fmt::FrontMask,
            Fmt::FrontLegend,
            Fmt::BackCopper,
            Fmt::BackMask,
            Fmt::BackLegend,
            Fmt::PlatedDrill,
            Fmt::NonPlatedDrill,
        ]
    }

    fn file_suffix(&self) -> &'static str {
        match self {
            Fmt::Edge => "Edge.Cuts.gm1",
            Fmt::FrontCopper => "F.Cu.gtl",
            Fmt::FrontMask => "F.Mask.gts",
            Fmt::FrontLegend => "F.SilkS.gto",
            Fmt::BackCopper => "B.Cu.gbl",
            Fmt::BackMask => "B.Mask.gbs",
            Fmt::BackLegend => "B.SilkS.gto",
            Fmt::PlatedDrill => "PTH.drl",
            Fmt::NonPlatedDrill => "NPTH.drl",
            Fmt::Zip => "gerbers.zip",
        }
    }

    fn serialize_to(&self, panel: &Panel, w: &mut impl std::io::Write) -> Result<(), Err> {
        match self {
            Fmt::Edge => panel.serialize_gerber_edges(w).map_err(|e| Err::General(e)),
            Fmt::FrontCopper => panel
                .serialize_gerber_layer(Layer::FrontCopper, w)
                .map_err(|e| Err::General(e)),
            Fmt::FrontMask => panel
                .serialize_gerber_layer(Layer::FrontMask, w)
                .map_err(|e| Err::General(e)),
            Fmt::FrontLegend => panel
                .serialize_gerber_layer(Layer::FrontLegend, w)
                .map_err(|e| Err::General(e)),
            Fmt::BackCopper => panel
                .serialize_gerber_layer(Layer::BackCopper, w)
                .map_err(|e| Err::General(e)),
            Fmt::BackMask => panel
                .serialize_gerber_layer(Layer::BackMask, w)
                .map_err(|e| Err::General(e)),
            Fmt::BackLegend => panel
                .serialize_gerber_layer(Layer::BackLegend, w)
                .map_err(|e| Err::General(e)),
            Fmt::PlatedDrill => panel.serialize_drill(w, true).map_err(|e| Err::IO(e)),
            Fmt::NonPlatedDrill => panel.serialize_drill(w, false).map_err(|e| Err::IO(e)),
            Fmt::Zip => {
                let mut cursor = std::io::Cursor::new(Vec::with_capacity(4 * 1024));
                let mut zip = zip::ZipWriter::new(&mut cursor);
                let options = zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored)
                    .unix_permissions(0o755);

                for fmt in Fmt::all_formats() {
                    zip.start_file(fmt.file_suffix(), options)
                        .map_err(|e| Err::Zip(e))?;
                    fmt.serialize_to(panel, &mut zip)?;
                }
                zip.finish().map_err(|e| Err::Zip(e))?;

                drop(zip);
                w.write(&cursor.into_inner()).map_err(|e| Err::IO(e))?;
                Ok(())
            }
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
            "drl" | "pdrl" => Ok(Fmt::PlatedDrill),
            "ndrl" | "npdrl" => Ok(Fmt::NonPlatedDrill),
            "zip" | "all" => Ok(Fmt::Zip),
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
    Gen {
        #[structopt(
            name = "fmt",
            short = "f",
            long = "fmt",
            about = "Specifies what output format to generate",
            default_value = "zip"
        )]
        fmt: Fmt,

        #[structopt(
            name = "output",
            short = "o",
            long = "output",
            about = "File path where the generated output should be written"
        )]
        output: Option<PathBuf>,
    },
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
                Rect::with_inner(ScrewHole::default()).bounds([0., 0.].into(), [5., 5.].into()),
                Direction::Right,
                8,
            ),
            repeating::Tile::new(
                Rect::with_inner(ScrewHole::default()).bounds([0., 0.].into(), [5., 5.].into()),
                Direction::Right,
                5,
            ),
            repeating::Tile::new(
                Rect::with_inner(ScrewHole::default()).bounds([0., 0.].into(), [5., 5.].into()),
                Direction::Right,
                8,
            ),
        ]),
        Some(Circle::wrap_with_radius(ScrewHole::with_diameter(5.), 7.5)),
        Some(Circle::wrap_with_radius(ScrewHole::with_diameter(5.), 7.5)),
    ));
    // panel.push(Circle::new([0., 7.5].into(), 7.5));
    // panel.push(Circle::new([20., 7.5].into(), 7.5));

    if let Err(e) = run_cmd(args, panel) {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    };
}

fn run_cmd(args: Opt, panel: Panel) -> Result<(), Err> {
    let mut stdout = std::io::stdout();

    match args.cmd {
        Cmd::Render { output, fit_to } => {
            let n = panel.make_svg().unwrap();
            // println!("{}", n.to_string(usvg::XmlOptions::default()));
            resvg::render_node(&n.root(), fit_to.0, Some(usvg::Color::white()))
                .unwrap()
                .save_png(output)
                .unwrap();
            Ok(())
        }
        Cmd::Gen { fmt, output: None } => fmt.serialize_to(&panel, &mut stdout),
        Cmd::Gen {
            fmt,
            output: Some(p),
        } => {
            let mut file = std::fs::File::create(&p).map_err(|e| Err::IO(e))?;
            fmt.serialize_to(&panel, &mut file)
        }
    }
}
