use std::ops::Range;
use pulldown_cmark::CowStr;
use serde::ser::{SerializeMap, SerializeStruct};
use serde::{Serialize, Serializer};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type Attributes<'a> = HashMap<CowStr<'a>, Value<'a>>;

#[derive(PartialEq, Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum Value<'a> {
  Hash(Attributes<'a>),
  Array(Vec<Value<'a>>),
  String(CowStr<'a>),
  Number(f64),
  Boolean(bool),
  #[serde(serialize_with = "serialize_variable")]
  Variable(char, Vec<Value<'a>>),
  #[serde(serialize_with = "serialize_function")]
  Function(CowStr<'a>, Attributes<'a>),
  Null,
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

#[derive(Clone, PartialEq, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Type {
  Document,
  Paragraph,
  Heading,
  Blockquote,
  Fence,
  List,
  Item,
  Inline,
  Table,
  #[serde(rename = "thead")]
  TableHead,
  #[serde(rename = "tbody")]
  TableBody,
  #[serde(rename = "tr")]
  TableRow,
  #[serde(rename = "th")]
  TableHeadCell,
  #[serde(rename = "td")]
  TableCell,
  #[serde(rename = "em")]
  Emphasis,
  Strong,
  Strike,
  Link,
  Image,
  Text,
  Code,
  SoftBreak,
  HardBreak,
  #[serde(rename = "hr")]
  Rule,
  Nop,
  Error,
  Tag(bool)
}

impl<'a> Default for Type {
  fn default() -> Self {
    Type::Nop
  }
}

impl Type {
  pub fn has_inline(&self) -> bool {
    use Type::*;
    matches!(self, Paragraph | Heading | Item | TableCell)
  }

  pub fn is_inline(&self) -> bool {
    use Type::*;
    match self {
      Emphasis | Strong | Strike | Link | Text | Code | SoftBreak | HardBreak => true,
      Tag(inline) => *inline,
      _ => false,
    }
  }
}

#[derive(PartialEq, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorLevel {
  Debug,
  Info,
  Warning,
  Error,
  Critical
}

#[derive(PartialEq, Debug, Serialize)]
pub struct Error {
  pub id: &'static str,
  pub level: ErrorLevel,
  pub message: String,
  pub location: Option<Range<usize>>
}

#[derive(PartialEq, Debug, Default)]
pub struct Node<'a> {
  pub kind: Type,
  pub tag: Option<CowStr<'a>>,
  pub attributes: Option<Attributes<'a>>,
  pub children: Option<Vec<NodeRef<'a>>>,
  pub location: Option<Range<usize>>,
  pub errors: Option<Vec<Error>>
}

impl<'a> Node<'a> {
  pub fn push(&mut self, child: NodeRef<'a>) {
    if let Some(children) = &mut self.children {
      children.push(child);
    } else {
      self.children = Some(vec![child]);
    }
  }

  pub fn attribute(&self, key: &str) -> Option<&Value> {
    self.attributes.as_ref().and_then(|x| x.get(key))
  }

  pub fn set_attributes(&mut self, attrs: HashMap<CowStr<'a>, Value<'a>>) {
    if let Some(attributes) = &mut self.attributes {
      attributes.extend(attrs)
    } else {
      self.attributes = Some(attrs)
    }
  }

  pub fn set_attribute(&mut self, key: CowStr<'a>, value: Value<'a>) -> Option<Value<'a>> {
    match &mut self.attributes {
      Some(attrs) => attrs.insert(key, value),
      None => {
        self.attributes = Some(Attributes::from([(key, value)]));
        None
      }
    }
  }
}

pub type NodeRef<'a> = Rc<RefCell<Node<'a>>>;

impl<'a> From<Node<'a>> for NodeRef<'a> {
  fn from(value: Node) -> NodeRef {
    Rc::new(RefCell::new(value))
  }
}

impl<'a> Serialize for Node<'a> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut field_count = 7;
    if let Type::Tag { .. } = &self.kind {
      field_count += 1;
    }

    let mut state = serializer.serialize_struct("Node", field_count)?;
    state.serialize_field("$$mdtype", "Node")?;
    state.serialize_field("inline", &self.kind.is_inline())?;
    state.serialize_field("location", &self.location)?;

    if let Type::Tag(..) = &self.kind {
      state.serialize_field("type", "tag")?;
      state.serialize_field("tag", &self.tag)?;
    } else {
      state.serialize_field("type", &self.kind)?;
    };

    if let Some(children) = &self.children {
      state.serialize_field("children", &children)?;
    } else {
      state.serialize_field("children", &Vec::<NodeRef<'a>>::new())?;
    }

    if let Some(errors) = &self.errors {
      state.serialize_field("errors", &errors)?;
    } else {
      state.serialize_field("errors", &Vec::<Error>::new())?;
    }

    if let Some(attrs) = &self.attributes {
      state.serialize_field("attributes", &attrs)?;
    } else {
      state.serialize_field("attributes", &Attributes::new())?;
    }

    state.end()
  }
}
