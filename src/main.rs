use maker_panel::{
    features::{repeating, AtPos, Circle, Column, Rect, ScrewHole},
    Direction, Err, Panel,
};

const DEFAULT_FIT: usvg::FitTo = usvg::FitTo::Zoom(23.);

fn main() {
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

    // let mut stdout = std::io::stdout();
    // panel.serialize_gerber_edges(&mut stdout).unwrap();
}
