use maker_panel::{Layer, Panel, SpecErr};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

mod version;

#[serde(remote = "SpecErr")]
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[wasm_bindgen]
pub fn render(spec: &str) -> JsValue {
    let mut panel = Panel::new();
    if let Err(e) = panel.push_spec(spec) {
        return JsValue::from_serde(&SpecErrHelper(e)).unwrap();
    }
    let edge = panel.edge_geometry();
    if edge.is_none() {
        return JsValue::from_serde(&Render {
            outer: vec![],
            inners: vec![],
        })
        .unwrap();
    }
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
        })
        .collect();

    JsValue::from_serde(&polys[0]).unwrap()
}
