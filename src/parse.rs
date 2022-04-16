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

  parse_tokens(&mut tokens, &mut nodes, false);

  root
}

fn append_to_parent(nodes: &mut Vec<NodeRef>, node: NodeRef) {
  if let Some(parent) = nodes.last_mut() {
    parent.borrow_mut().children.push(node);
  }
}

fn parse_tokens(tokens: &mut dyn Iterator<Item = Token>, nodes: &mut Vec<NodeRef>, in_fence: bool) {
  let mut inline: Option<NodeRef> = None;
  let mut fence: Option<String> = None;
  let mut thead = false;

  for token in tokens {
    // Concatenate text nodes for fence
    if let (
      Token::Append {
        kind: Type::Text,
        attributes: Some(attrs),
      },
      Some(content),
    ) = (&token, &mut fence)
    {
      if let Some(Value::String(value)) = attrs.get("content") {
        content.push_str(value);
      }
      continue;
    }

    // todo(compat): Combine successive text nodes 
    if let Some(parent) = nodes.last() {
      if let Some(last) = parent.borrow().children.last() {
        let mut node = last.borrow_mut();

        if let (
          Node {
            kind: Type::Text,
            attributes: Some(existing_attrs),
            ..
          },
          Token::Append {
            kind: Type::Text,
            attributes: Some(new_attrs),
          },
        ) = (&*node, &token)
        {
          if let (Some(Value::String(existing_content)), Some(Value::String(new_content))) =
            (existing_attrs.get("content"), new_attrs.get("content"))
          {
            if new_content != " " {
              let updated_content = format!("{}{}", existing_content, new_content);
              node.set_attribute("content", updated_content.into());
              continue;
            }
          }
        }
      }
    }

    if !in_fence { // todo(compat): remove fence condition
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
    }

    match token {
      // Parse tags inside of fenced code block
      Token::Close { kind: Type::Fence } => {
        if let Some(content) = fence {
          parse_tokens(&mut template::parse(&content).into_iter(), nodes, true);
          if let Some(parent) = nodes.pop() {
            parent.borrow_mut().set_attribute("content", content.into());
          }
        }

        fence = None;
      }

      // Table normalization logic
      Token::Open {
        kind: Type::TableHead,
        attributes,
      } => {
        thead = true;

        let node: NodeRef = Node::new(Type::TableHead, attributes).into();
        append_to_parent(nodes, node.clone());
        nodes.push(node);

        let node: NodeRef = Node::new(Type::TableRow, None).into();
        append_to_parent(nodes, node.clone());
        nodes.push(node);
      }

      Token::Close {
        kind: Type::TableHead,
      } => {
        thead = false;
        nodes.pop();
        nodes.pop();

        let node: NodeRef = Node::new(Type::TableBody, None).into();
        append_to_parent(nodes, node.clone());
        nodes.push(node);
      }

      Token::Close { kind: Type::Table } => {
        nodes.pop();
        nodes.pop();
      }

      Token::Open {
        kind: Type::TableCell,
        attributes,
      } => {
        let kind = if thead {
          Type::TableHeadCell
        } else {
          Type::TableCell
        };

        let node: NodeRef = Node::new(kind, attributes).into();
        append_to_parent(nodes, node.clone());
        nodes.push(node);
      }

      Token::Close {
        kind: Type::TableCell,
      } => {
        let kind = if thead {
          Type::TableHeadCell
        } else {
          Type::TableCell
        };

        if let Some(parent) = nodes.last() {
          if parent.borrow().kind == kind {
            nodes.pop();
          }
        }
      }

      // Standard token handlers
      Token::Open { kind, attributes } => {
        if kind == Type::Fence {
          fence = Some(String::new());
        }

        let node: NodeRef = Node::new(kind, attributes).into();
        append_to_parent(nodes, node.clone());
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
        // Ignore empty tokens
        if let Some(attrs) = &attributes {
          if let Some(Value::String(content)) = attrs.get("content") {
            if content.is_empty() {
              continue;
            }
          }
        }

        let node = Node::new(kind, attributes);
        append_to_parent(nodes, node.into());
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

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

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
            kind: Type::Text,
            attributes: Some([("content".into(), "This is a test\n".into())].into()),
            children: vec![]
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
