use maker_panel::{features::InnerAtom, Layer, Panel, SpecErr};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

mod version;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(remote = "SpecErr")]
pub enum Err {
    Parse(String),
    UndefinedVariable(String),
    BadType(String),
}

#[derive(Serialize, Deserialize)]
struct SpecErrHelper(#[serde(with = "Err")] SpecErr);

#[wasm_bindgen]
pub fn check_parse_err(spec: &str) -> JsValue {
    let mut panel = Panel::new();
    if let Err(e) = panel.push_spec(spec) {
        JsValue::from_serde(&SpecErrHelper(e)).unwrap()
    } else {
        JsValue::undefined()
    }
}

#[wasm_bindgen]
pub fn maker_panel_version() -> String {
    version::MP_VERSION.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Render {
    pub outer: Vec<(f64, f64)>,
    pub inners: Vec<Vec<(f64, f64)>>,
    pub surface_features: Vec<Surface>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Surface {
    Drill {
        center: (f64, f64),
        radius: f64,
        plated: bool,
    },
    Circle {
        center: (f64, f64),
        radius: f64,
        layer: String,
    },
}

impl std::convert::TryFrom<&InnerAtom> for Surface {
    type Error = ();

    fn try_from(a: &InnerAtom) -> Result<Self, Self::Error> {
        match a {
            InnerAtom::Drill {
                center,
                radius,
                plated,
            } => Ok(Surface::Drill {
                radius: *radius,
                plated: *plated,
                center: (center.x_y().0, center.x_y().1),
            }),
            InnerAtom::Circle {
                center,
                radius,
                layer,
            } => Ok(Surface::Circle {
                radius: *radius,
                layer: layer.to_string(),
                center: (center.x_y().0, center.x_y().1),
            }),
            _ => Err(()),
        }
    }
}

#[wasm_bindgen]
pub fn render(spec: &str, convex_hull: bool) -> JsValue {
    let mut panel = Panel::new();
    if let Err(e) = panel.push_spec(spec) {
        return JsValue::from_serde(&SpecErrHelper(e)).unwrap();
    }
    panel.convex_hull(convex_hull);
    let edge = panel.edge_geometry();
    if edge.is_none() {
        return JsValue::from_serde(&Render {
            outer: vec![],
            inners: vec![],
            surface_features: vec![],
        })
        .unwrap();
    }

    use std::convert::TryFrom;
    let polys: Vec<_> = edge
        .unwrap()
        .iter()
        .map(|p| Render {
            outer: p.exterior().points_iter().map(|p| p.x_y()).collect(),
            inners: p
                .interiors()
                .iter()
                .map(|l| l.points_iter().map(|p| p.x_y()).collect())
                .collect(),
            surface_features: panel
                .interior_geometry()
                .iter()
                .map(|f| Surface::try_from(f))
                .filter(|f| f.is_ok())
                .map(|f| f.unwrap())
                .collect(),
        })
        .collect();

    JsValue::from_serde(&polys[0]).unwrap()
}
