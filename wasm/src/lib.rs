use markdoc::parse;
use serde_json;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse(input: &str) -> String {
    let doc = parse::parse(input);
    serde_json::to_string(&doc).unwrap()
}