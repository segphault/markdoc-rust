use super::node::*;
use super::render::Renderable;
use pulldown_cmark::CowStr;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

#[derive(Debug, Serialize, Deserialize)]
pub enum SchemaType {
  String,
  Number,
  Boolean,
  Object,
  Array,
}

#[derive(Debug, Serialize)]
pub enum AttributeRender<'a> {
  Name(&'a str),
  True,
  False,
}

impl<'a> Default for AttributeRender<'a> {
  fn default() -> Self {
    AttributeRender::True
  }
}

#[derive(Debug, Serialize, Default)]
pub struct Attribute<'a> {
  #[serde(default)]
  pub kind: Option<SchemaType>,
  #[serde(default)]
  pub render: AttributeRender<'a>,
  #[serde(default)]
  pub required: bool,
}

pub type TransformFn<'a> = fn(&Node<'a>, &'a Config<'a>) -> Renderable<'a>;

pub enum Transform<'a> {
  Function(TransformFn<'a>),
  Template(CowStr<'a>),
}

impl<'a> fmt::Debug for Transform<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Transform::Function(..) => f.write_str("[FUNCTION]"),
      Transform::Template(..) => f.write_str("[TEMPLATE]")
    }
  }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Schema<'a> {
  #[serde(default)]
  pub render: Option<&'a str>,
  #[serde(skip)]
  pub attributes: Option<HashMap<&'a str, Attribute<'a>>>,
  #[serde(default)]
  pub self_closing: bool,
  #[serde(skip)]
  pub transform: Option<Transform<'a>>,
}

#[derive(Serialize, Deserialize)]
pub struct Config<'a> {
  #[serde(borrow)]
  pub nodes: HashMap<Type, Schema<'a>>,
  #[serde(borrow)]
  pub tags: HashMap<&'a str, Schema<'a>>,
}
