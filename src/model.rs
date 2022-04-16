use crate::markdown::CowStr;
use serde::ser::{SerializeMap, SerializeStruct};
use serde::{Serialize, Serializer};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type Attributes = HashMap<String, Value>;

#[derive(PartialEq, Clone, Debug, Serialize)]
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

impl From<&str> for Value {
  fn from(value: &str) -> Value {
    Value::String(value.into())
  }
}

impl From<CowStr<'_>> for Value {
  fn from(value: CowStr) -> Value {
    Value::String(value.to_string())
  }
}

impl From<&String> for Value {
  fn from(value: &String) -> Value {
    Value::String(value.to_string())
  }
}

impl From<String> for Value {
  fn from(value: String) -> Value {
    Value::String(value.to_string())
  }
}

impl From<bool> for Value {
  fn from(value: bool) -> Value {
    Value::Boolean(value)
  }
}

impl From<i32> for Value {
  fn from(value: i32) -> Value {
    Value::Number(value.into())
  }
}

impl From<u64> for Value {
  fn from(value: u64) -> Value {
    Value::Number(value as f64)
  }
}

impl From<u8> for Value {
  fn from(value: u8) -> Value {
    Value::Number(value as f64)
  }
}

impl From<Vec<Value>> for Value {
  fn from(value: Vec<Value>) -> Value {
    Value::Array(value)
  }
}

impl<const N: usize> From<[(&str, Value); N]> for Value {
  fn from(value: [(&str, Value); N]) -> Value {
    Value::Hash(value.map(|(k, v)| (k.into(), v)).into())
  }
}

fn serialize_variable<S>(ch: &char, path: &[Value], s: S) -> Result<S::Ok, S::Error>
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

#[derive(Serialize, Debug, PartialEq)]
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

impl Token {
  pub fn kind(&self) -> Option<&Type> {
    use Token::*;
    match self {
      Open { kind, .. } | Append { kind, .. } | Close {kind} => Some(kind),
      _ => None
    }
  }

  pub fn is_inline(&self) -> bool {
    use Token::*;
    match self {
      Open { kind, .. } | Append { kind, .. } | Close { kind } => kind.is_inline(),
      Annotate { .. } => true,
      _ => false,
    }
  }

  pub fn attribute(&self, key: &str) -> Option<&Value> {
    use Token::*;
    match self {
      Open { attributes, .. } | Append { attributes, .. } | Annotate { attributes, .. } => {
        attributes.as_ref().and_then(|x| x.get(key))
      }
      _ => None,
    }
  }
}

#[derive(PartialEq, Debug)]
pub struct Node {
  pub kind: Type,
  pub attributes: Option<Attributes>,
  pub children: Vec<NodeRef>,
}

impl Node {
  pub fn new(kind: Type, attributes: Option<Attributes>) -> Self {
    Node {
      kind,
      attributes,
      children: Vec::new(),
    }
  }

  pub fn attribute(&self, key: &str) -> Option<&Value> {
    self.attributes.as_ref().and_then(|x| x.get(key))
  }

  pub fn set_attributes(&mut self, attrs: Attributes) {
    if let Some(attributes) = &mut self.attributes {
      attributes.extend(attrs)
    } else {
      self.attributes = Some(attrs)
    }
  }

  pub fn set_attribute(&mut self, key: &str, value: Value) -> Option<Value> {
    match &mut self.attributes {
      Some(attrs) => attrs.insert(key.into(), value),
      None => {
        self.attributes = Some(Attributes::from([(key.into(), value)]));
        None
      }
    }
  }
}

pub type NodeRef = Rc<RefCell<Node>>;

impl From<Node> for NodeRef {
  fn from(value: Node) -> NodeRef {
    Rc::new(RefCell::new(value))
  }
}

impl Serialize for Node {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut field_count = 5;
    if let Type::Tag { .. } = &self.kind {
      field_count += 1;
    }

    let mut state = serializer.serialize_struct("Node", field_count)?;
    state.serialize_field("$$mdtype", "Node")?;
    state.serialize_field("children", &self.children)?;
    state.serialize_field("inline", &self.kind.is_inline())?;

    if let Type::Tag { name, .. } = &self.kind {
      state.serialize_field("type", "tag")?;
      state.serialize_field("tag", name)?;
    } else {
      state.serialize_field("type", &self.kind)?;
    };

    if let Some(attrs) = &self.attributes {
      state.serialize_field("attributes", &attrs)?;
    } else {
      state.serialize_field("attributes", &Attributes::new())?;
    }

    state.end()
  }
}
