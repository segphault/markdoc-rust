use super::render::Renderable;
use super::Attributes;
use super::{node::*, value::Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub enum SchemaType {
  String,
  Number,
  Boolean,
  Object,
  Array,
}

#[derive(Serialize)]
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

#[derive(Serialize, Deserialize, Default)]
pub struct Attribute<'a> {
  #[serde(default)]
  pub kind: Option<SchemaType>,
  #[serde(skip)]
  pub render: AttributeRender<'a>,
  #[serde(default)]
  pub required: bool,
}

pub type TransformFn<'a> = fn(&Node<'a>, &'a Config<'a>) -> Renderable<'a>;

#[derive(Default, Serialize, Deserialize)]
pub struct Schema<'a> {
  #[serde(default)]
  pub render: Option<&'a str>,
  #[serde(default)]
  pub attributes: Option<HashMap<&'a str, Attribute<'a>>>,
  #[serde(default)]
  pub self_closing: bool,
  #[serde(skip)]
  pub transform: Option<TransformFn<'a>>,
}

pub type EvaluateFn<'a> = fn(&Attributes<'a>, &'a Config<'a>) -> Value<'a>;

pub struct FunctionSchema<'a> {
  pub attributes: Option<HashMap<&'a str, Attribute<'a>>>,
  pub evaluate: EvaluateFn<'a>,
}

pub type VariableFn<'a> = fn(&[Value<'a>]) -> Value<'a>;
pub enum Variables<'a> {
  Resolver(VariableFn<'a>),
  Values(Attributes<'a>),
}

#[derive(Serialize, Deserialize)]
pub struct Config<'a> {
  #[serde(borrow)]
  pub nodes: HashMap<Type, Schema<'a>>,
  #[serde(borrow)]
  pub tags: HashMap<&'a str, Schema<'a>>,
  #[serde(skip)]
  pub variables: Option<Variables<'a>>,
  #[serde(skip)]
  pub functions: Option<HashMap<&'a str, FunctionSchema<'a>>>,
}
