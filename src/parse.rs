use crate::model::{Attributes, Node, NodeRef, Token, Type};
use crate::tokenize::tokenize;

pub fn extract_frontmatter(input: &str) -> (&str, Option<Attributes>) {
  if let Some((frontmatter, text)) = input
    .strip_prefix("---")
    .and_then(|left| left.split_once("---"))
  {
    (
      text,
      Some([("frontmatter".into(), frontmatter.into())].into()),
    )
  } else {
    (input, None)
  }
}

pub fn parse(input: &str) -> NodeRef {
  let (source, attrs) = extract_frontmatter(input);
  let root: NodeRef = Node::new(Type::Document, attrs).into();
  let mut nodes = vec![root.clone()];
  let mut inline: Option<NodeRef> = None;

  for token in tokenize(source) {
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

    match token {
      Token::Open { kind, attributes } => {
        let node: NodeRef = Node::new(kind, attributes).into();
        if let Some(parent) = nodes.last_mut() {
          parent.borrow_mut().children.push(node.clone());
        }

        nodes.push(node);
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
          let node = Node::new(kind, attributes);

          // Special case handling for content in fenced code blocks
          if let Some(inline_ref) = &inline {
            let mut inline_node = inline_ref.borrow_mut();
            if inline_node.kind == Type::Fence {
              // Apply the text content of a fenced code block to the content attribute
              if let Some(value) = node.attribute("content") {
                inline_node.set_attribute("content", value.clone());
              }
            }
          }

          parent.borrow_mut().children.push(node.into());
        }
      }

      Token::Annotate { attributes } => {
        if let Some(node) = &inline {
          if let Some(attributes) = attributes {
            node.borrow_mut().set_attributes(attributes);
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

  #[test]
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
