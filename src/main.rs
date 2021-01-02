use maker_panel::{features::Rect, make_svg, Err, PanelBuilder};

const DEFAULT_FIT: usvg::FitTo = usvg::FitTo::Width(450);

fn main() {
    let mut panel = PanelBuilder::new();
    panel.push(Rect::new_with_center([-2.5, -2.5].into(), 5., 5.));
    panel.push(Rect::new([-0., -1.].into(), [5., 3.].into()));

    println!("panel: {}", panel);
    println!("edges: {:?}", panel.edge_geometry());

    let n = make_svg(panel.edge_geometry().unwrap()).unwrap();

    // println!("{}", n.to_string(usvg::XmlOptions::default()));
    resvg::render_node(&n.root(), DEFAULT_FIT, Some(usvg::Color::white()))
        .unwrap()
        .save_png("/tmp/ye.png")
        .unwrap();
}
