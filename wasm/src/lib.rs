use markdoc::model::{Token};
use markdoc::parse;
use markdoc::tokenize;
use serde_json;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse(input: &str) -> String {
    let doc = parse::parse(input);
    serde_json::to_string(&doc).unwrap()
}

#[wasm_bindgen]
pub fn raw_tokens(input: &str) -> String {
    let (source, attrs) = parse::extract_frontmatter(input);
    let tokens: Vec<Token> = tokenize::tokenize(&source).collect();
    serde_json::to_string(&tokens).unwrap()
}