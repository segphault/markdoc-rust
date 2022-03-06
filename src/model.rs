use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type Attributes = HashMap<String, Value>;

#[derive(PartialEq, Debug, Serialize)]
#[serde(untagged)]
pub enum Value {
  Hash(Attributes),
  Array(Vec<Value>),
  String(String),
  Number(f64),
  Boolean(bool),
  #[serde(serialize_with = "serialize_variable")]
  Variable(char, Vec<Value>),
  #[serde(serialize_with = "serialize_function")]
  Function(String, Attributes),
  Null,
}

fn serialize_variable<S>(ch: &char, path: &Vec<Value>, s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let mut map = s.serialize_map(Some(3))?;
  map.serialize_entry("$$mdtype", "Variable")?;
  map.serialize_entry("prefix", ch)?;
  map.serialize_entry("path", &path)?;
  map.end()
}

fn serialize_function<S>(name: &String, attrs: &Attributes, s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let mut map = s.serialize_map(Some(3))?;
  map.serialize_entry("$$mdtype", "Function")?;
  map.serialize_entry("name", &name)?;
  map.serialize_entry("parameters", &attrs)?;
  map.end()
}

#[derive(PartialEq, Debug, Serialize)]
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
  #[serde(rename = "tr")]
  TableRow,
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
  Tag {
    inline: bool,
    name: String,
  },
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
      Tag { inline, .. } => *inline,
      _ => false,
    }
  }
}

#[derive(Serialize)]
#[serde(tag = "operation", content = "token")]
pub enum Token {
  Open {
    #[serde(rename = "type")]
    kind: Type,
    attributes: Option<Attributes>,
  },
  Close {
    #[serde(rename = "type")]
    kind: Type,
  },
  Append {
    #[serde(rename = "type")]
    kind: Type,
    attributes: Option<Attributes>,
  },
  Annotate {
    attributes: Option<Attributes>,
  },
  Error {
    message: String,
  },
}

#[derive(PartialEq, Debug, Serialize)]
#[serde(tag = "$$mdtype")]
pub struct Node {
  #[serde(rename = "type")]
  pub kind: Type,
  pub attributes: Option<Attributes>,
  pub children: Vec<Rc<RefCell<Node>>>,
}

impl Node {
  pub fn new(kind: Type, attributes: Option<Attributes>) -> Self {
    Node {
      kind,
      attributes,
      children: Vec::new(),
    }
  }
}
