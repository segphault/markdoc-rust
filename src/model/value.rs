use super::Attributes;
use pulldown_cmark::CowStr;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

pub type ValueRef<'a> = Rc<RefCell<Value<'a>>>;
#[derive(PartialEq, Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum Value<'a> {
  Hash(Attributes<'a>),
  Array(Vec<Value<'a>>),
  String(CowStr<'a>),
  Number(f64),
  Boolean(bool),
  Expression(ValueRef<'a>, Expression<'a>),
  Undefined,
  Null,
}

impl<'a> Value<'a> {
  pub fn resolved<T>(&self, f: fn (&Value<'a>) -> T) -> T {
    match self {
      Value::Expression(value, _) => f(&value.borrow()),
      _ => f(self)
    }
  }

  pub fn deep_get(&'a self, path: &[Value<'a>]) -> Option<&'a Value<'a>> {
    path
      .into_iter()
      .fold(Some(self), |cur, iter| match (cur, iter) {
        (Some(Value::Hash(items)), Value::String(key)) => items.get(key),
        (Some(Value::Array(items)), Value::Number(idx)) => items.get(*idx as usize),
        _ => None,
      })
  }
}

impl<'a> From<&'a str> for Value<'a> {
  fn from(value: &'a str) -> Value<'a> {
    Value::String(value.into())
  }
}

impl<'a> From<CowStr<'a>> for Value<'a> {
  fn from(value: CowStr<'a>) -> Value<'a> {
    Value::String(value)
  }
}

impl<'a> From<String> for Value<'a> {
  fn from(value: String) -> Value<'a> {
    Value::String(value.into())
  }
}

impl<'a> From<bool> for Value<'a> {
  fn from(value: bool) -> Value<'a> {
    Value::Boolean(value)
  }
}

impl<'a> From<i32> for Value<'a> {
  fn from(value: i32) -> Value<'a> {
    Value::Number(value.into())
  }
}

impl<'a> From<Vec<Value<'a>>> for Value<'a> {
  fn from(value: Vec<Value>) -> Value {
    Value::Array(value)
  }
}

impl<'a> From<Attributes<'a>> for Value<'a> {
  fn from(value: Attributes) -> Value {
    Value::Hash(value)
  }
}

impl<'a> From<Expression<'a>> for Value<'a> {
  fn from(value: Expression) -> Value {
    Value::Expression(Rc::new(RefCell::new(Value::Undefined)), value)
  }
}

impl<'a> fmt::Display for Value<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Value::String(value) => write!(f, "{}", value),
      Value::Number(value) => write!(f, "{}", value),
      Value::Boolean(value) => write!(f, "{}", value),
      Value::Expression(value, _) => write!(f, "{}", value.borrow_mut()),
      _ => write!(f, "[OBJECT]"),
    }
  }
}

#[derive(PartialEq, Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum Expression<'a> {
  #[serde(serialize_with = "serialize_variable")]
  Variable(char, Vec<Value<'a>>),
  #[serde(serialize_with = "serialize_function")]
  Function(CowStr<'a>, Attributes<'a>),
}

fn serialize_variable<S>(_ch: &char, path: &[Value], s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let mut map = s.serialize_map(Some(3))?;
  map.serialize_entry("$$mdtype", "Variable")?;
  map.serialize_entry("path", &path)?;
  map.end()
}

fn serialize_function<S>(name: &str, attrs: &Attributes, s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let mut map = s.serialize_map(Some(3))?;
  map.serialize_entry("$$mdtype", "Function")?;
  map.serialize_entry("name", &name)?;
  map.serialize_entry("parameters", &attrs)?;
  map.end()
}
