use maker_panel::{
    features::{repeating, Rect, ScrewHole},
    Direction, Err, Panel,
};

const DEFAULT_FIT: usvg::FitTo = usvg::FitTo::Zoom(23.);

fn main() {
    let mut panel = Panel::new();
    panel.convex_hull(true);
    // panel.push(Rect::with_center([0.0, -2.5].into(), 5., 5.));
    panel.push(repeating::Tile::new(
        Rect::with_inner(ScrewHole::default(), [0., 0.].into(), [5., 5.].into()),
        Direction::Right,
        3,
    ));
    panel.push(repeating::Tile::new(
        Rect::with_inner(ScrewHole::default(), [2.5, 5.].into(), [7.5, 10.].into()),
        Direction::Right,
        2,
    ));
    panel.push(repeating::Tile::new(
        Rect::with_inner(ScrewHole::default(), [0., 10.].into(), [5., 15.].into()),
        Direction::Right,
        3,
    ));

    let n = panel.make_svg().unwrap();

    println!("{}", n.to_string(usvg::XmlOptions::default()));
    resvg::render_node(&n.root(), DEFAULT_FIT, Some(usvg::Color::white()))
        .unwrap()
        .save_png("/tmp/ye.png")
        .unwrap();
}
