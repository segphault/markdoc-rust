use crate::model::{Attributes, Node, Token, Type, Value};
use crate::tokenize::tokenize;
use std::cell::RefCell;
use std::rc::Rc;

pub fn extract_frontmatter(input: &str) -> (&str, Option<Attributes>) {
  match input
    .strip_prefix("---")
    .and_then(|left| left.split_once("---"))
  {
    Some((frontmatter, text)) => (
      text,
      Some(Attributes::from([(
        "frontmatter".into(),
        Value::String(frontmatter.into()),
      )])),
    ),
    None => (input, None),
  }
}

pub fn parse(input: &str) -> Rc<RefCell<Node>> {
  let (source, attrs) = extract_frontmatter(input);
  let root = Rc::new(RefCell::new(Node::new(Type::Document, attrs)));
  let mut nodes = vec![root.clone()];
  let mut inline: Option<Rc<RefCell<Node>>> = None;

  for token in tokenize(source) {
    if token.is_inline() && inline.is_none() {
      let inline_node = Rc::new(RefCell::new(Node::new(Type::Inline, None)));
      if let Some(parent) = nodes.last_mut() {
        inline = Some(parent.clone());
        parent.borrow_mut().children.push(inline_node.clone());
      }
      nodes.push(inline_node);
    }

    if let Some(parent) = nodes.last() {
      if !token.is_inline() && inline.is_some() && parent.borrow().kind == Type::Inline {
        inline = None;
        nodes.pop();
      }
    } 

    match token {
      Token::Open { kind, attributes } => {
        let node = Rc::new(RefCell::new(Node::new(kind, attributes)));

        if let Some(parent) = nodes.last_mut() {
          parent.borrow_mut().children.push(node.clone());
        }

        nodes.push(node.clone());
      }

      Token::Close { kind } => {
        if let Some(parent) = nodes.last() {
          if parent.borrow().kind == kind {
            nodes.pop();
          }
        }
      }

      Token::Append { kind, attributes } => {
        if let Some(parent) = nodes.last_mut() {
          let node = Rc::new(RefCell::new(Node::new(kind, attributes)));
          parent.borrow_mut().children.push(node);
        }
      }

      Token::Annotate { attributes } => {
        if let Some(node) = &inline {
          if let Some(attributes) = attributes {
            if let Some(attrs) = &mut node.borrow_mut().attributes {
              attrs.extend(attributes);
            } else {
              node.borrow_mut().attributes = Some(attributes);
            }
          }
        }
      }

      _ => (),
    }
  }

  root
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::model::{Attributes, Value};

  #[test]
  fn basic_parse() {
    let output = parse("# Heading");
    assert_eq!(
      output,
      Rc::new(RefCell::new(Node {
        kind: Type::Document,
        attributes: None,
        children: vec![Rc::new(RefCell::new(Node {
          kind: Type::Heading,
          attributes: Some(Attributes::from([("level".into(), Value::Number(1.0))])),
          children: vec![Rc::new(RefCell::new(Node {
            kind: Type::Inline,
            attributes: None,
            children: vec![Rc::new(RefCell::new(Node {
              kind: Type::Text,
              attributes: Some([("content".into(), Value::String("Heading".to_string()))].into()),
              children: vec![]
            }))]
          }))]
        }))]
      }))
    )
  }

  #[test]
  fn parse_heading_with_annotation() {
    let output = parse("# Heading {% foo=true %}");
    assert_eq!(
      output,
      Rc::new(RefCell::new(Node {
        kind: Type::Document,
        attributes: None,
        children: vec![Rc::new(RefCell::new(Node {
          kind: Type::Heading,
          attributes: Some(Attributes::from([
            ("level".into(), Value::Number(1.0)),
            ("foo".into(), Value::Boolean(true))
          ])),
          children: vec![Rc::new(RefCell::new(Node {
            kind: Type::Inline,
            attributes: None,
            children: vec![Rc::new(RefCell::new(Node {
              kind: Type::Text,
              attributes: Some(Attributes::from([(
                "content".into(),
                Value::String("Heading ".into())
              )])),
              children: vec![]
            }))]
          }))]
        }))]
      }))
    )
  }
}
