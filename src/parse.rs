use crate::model::{Attributes, Node, NodeRef, Token, Type, Value};
use crate::template;
use crate::tokenize::tokenize;

pub fn extract_frontmatter(input: &str) -> (&str, Option<Attributes>) {
  if let Some((frontmatter, text)) = input
    .strip_prefix("---")
    .and_then(|left| left.split_once("---"))
  {
    (
      text,
      Some([("frontmatter".into(), frontmatter.trim().into())].into()),
    )
  } else {
    (input, None)
  }
}

pub fn parse(input: &str) -> NodeRef {
  let (source, attrs) = extract_frontmatter(input);
  let root: NodeRef = Node::new(Type::Document, attrs).into();
  let mut tokens = tokenize(source);
  let mut nodes = vec![root.clone()];

  parse_tokens(&mut tokens, &mut nodes);

  root
}

fn parse_tokens(tokens: &mut dyn Iterator<Item = Token>, nodes: &mut Vec<NodeRef>) {
  let mut inline: Option<NodeRef> = None;

  for token in tokens {
    if let Some(parent) = nodes.last() {
      if parent.borrow().kind == Type::Fence && parent.borrow().attribute("content").is_none() {
        if let Some(Value::String(value)) = token.attribute("content") {
          parent.borrow_mut().set_attribute("content", value.into());
          parse_tokens(&mut template::parse(value).into_iter(), nodes);
        }

        continue;
      }
    }

    // When adding an inline node, insert an inline-type block node if one isn't already present
    if token.is_inline() && inline.is_none() {
      let inline_node: NodeRef = Node::new(Type::Inline, None).into();
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

    if let Some(parent) = nodes.last_mut() {
      match token {
        Token::Open { kind, attributes } => {
          let node: NodeRef = Node::new(kind, attributes).into();
          parent.borrow_mut().children.push(node.clone());
          nodes.push(node);
        }

        Token::Close { kind } => {
          if parent.borrow().kind == kind {
            nodes.pop();
          }
        }

        Token::Append { kind, attributes } => {
          let node = Node::new(kind, attributes);
          parent.borrow_mut().children.push(node.into());
        }

        Token::Annotate { attributes } => {
          if let (Some(node), Some(attributes)) = (&inline, attributes) {
            node.borrow_mut().set_attributes(attributes);
          }
        }

        _ => (),
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fence_with_info_string() {
    let output = parse("```javascript {% foo=2 %}\nThis is a test\n```");

    assert_eq!(
      output,
      Node {
        kind: Type::Document,
        attributes: None,
        children: vec![Node {
          kind: Type::Fence,
          attributes: Some(
            [
              ("foo".into(), 2.into()),
              ("content".into(), "This is a test\n".into()),
              ("language".into(), "javascript".into()),
            ]
            .into()
          ),
          children: vec![Node {
            kind: Type::Inline,
            attributes: None,
            children: vec![Node {
              kind: Type::Text,
              attributes: Some([("content".into(), "This is a test\n".into())].into()),
              children: vec![]
            }
            .into()]
          }
          .into()]
        }
        .into()]
      }
      .into()
    )
  }

  #[test]
  fn basic_parse() {
    let output = parse("# Heading");
    assert_eq!(
      output,
      Node {
        kind: Type::Document,
        attributes: None,
        children: vec![Node {
          kind: Type::Heading,
          attributes: Some([("level".into(), 1.into())].into()),
          children: vec![Node {
            kind: Type::Inline,
            attributes: None,
            children: vec![Node {
              kind: Type::Text,
              attributes: Some([("content".into(), "Heading".into())].into()),
              children: vec![]
            }
            .into()]
          }
          .into()]
        }
        .into()]
      }
      .into()
    )
  }

  // #[test]
  fn parse_heading_with_annotation() {
    let output = parse("# Heading {% foo=true %}");
    assert_eq!(
      output,
      Node {
        kind: Type::Document,
        attributes: None,
        children: vec![Node {
          kind: Type::Heading,
          attributes: Some([("level".into(), 1.into()), ("foo".into(), true.into())].into()),
          children: vec![Node {
            kind: Type::Inline,
            attributes: None,
            children: vec![Node {
              kind: Type::Text,
              attributes: Some([("content".into(), "Heading ".into())].into()),
              children: vec![]
            }
            .into()]
          }
          .into()]
        }
        .into()]
      }
      .into()
    )
  }
}
