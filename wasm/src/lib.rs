use maker_panel::{Layer, Panel, SpecErr};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

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
