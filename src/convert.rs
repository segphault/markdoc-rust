use crate::markdown;
use crate::model::{Attributes, Token, Type, Value};
use crate::tag::Tag;
use std::collections::HashMap;

impl<'a> From<&'a markdown::Event<'a>> for Type {
  fn from(event: &'a markdown::Event<'a>) -> Type {
    use markdown::Event::*;
    use markdown::Tag::*;

    match event {
      End(tag) | Start(tag) => match tag {
        Paragraph => Type::Paragraph,
        Heading(..) => Type::Heading,
        BlockQuote => Type::Blockquote,
        CodeBlock(..) => Type::Fence,
        List(..) => Type::List,
        Item => Type::Item,
        Table(..) => Type::Table,
        TableHead => Type::TableHead,
        TableRow => Type::TableRow,
        TableCell => Type::TableCell,
        Emphasis => Type::Emphasis,
        Strong => Type::Strong,
        Strikethrough => Type::Strike,
        Link(..) => Type::Link,
        Image(..) => Type::Image,
        _ => Type::Nop,
      },
      Text(..) => Type::Text,
      Code(..) => Type::Code,
      SoftBreak => Type::SoftBreak,
      HardBreak => Type::HardBreak,
      Rule => Type::Rule,
      _ => Type::Nop,
    }
  }
}

impl From<markdown::Event<'_>> for Token {
  fn from(event: markdown::Event) -> Token {
    use markdown::Event::*;
    let kind = Type::from(&event);

    match event {
      Start(_) => Token::Open {
        kind,
        attributes: convert_attributes(event),
      },
      End(_) => Token::Close { kind },
      MarkdocTag(text, inline) => {
        let parsed_tag = Tag::from(text.as_ref());
        match parsed_tag {
          Tag::Standalone(name, attributes) => Token::Append {
            kind: Type::Tag {
              name: String::from(name),
              inline,
            },
            attributes,
          },
          Tag::Open(name, attributes) => Token::Open {
            kind: Type::Tag {
              name: String::from(name),
              inline,
            },
            attributes,
          },
          Tag::Close(name) => Token::Close {
            kind: Type::Tag {
              name: String::from(name),
              inline,
            },
          },
          Tag::Value(variable) => Token::Append {
            kind: Type::Text,
            attributes: Some(Attributes::from([(String::from("content"), variable)])),
          },
          Tag::Annotation(attributes) => Token::Annotate {
            attributes: Some(attributes),
          },
          Tag::Error(error) => Token::Error {
            message: error.to_string(),
          },
        }
      }
      _ => Token::Append {
        kind,
        attributes: convert_attributes(event),
      },
    }
  }
}

pub fn convert_attributes(event: markdown::Event) -> Option<Attributes> {
  use markdown::CodeBlockKind::*;
  use markdown::Event::*;
  use markdown::Tag::*;

  match event {
    End(tag) | Start(tag) => match tag {
      Heading(level, ..) => Some(HashMap::from([(
        String::from("level"),
        Value::Number(level as u8 as f64),
      )])),
      CodeBlock(kind) => match kind {
        Fenced(info) => Some(HashMap::from([(
          String::from("language"),
          Value::String(info.to_string()),
        )])),
        Indented => None,
      },
      List(ordered) => match ordered {
        Some(n) => Some(HashMap::from([
          (String::from("ordered"), Value::Boolean(true)),
          (String::from("number"), Value::Number(n as f64)),
        ])),
        None => Some(HashMap::from([(
          String::from("ordered"),
          Value::Boolean(false),
        )])),
      },
      Link(_, link, title) => Some(HashMap::from([
        (String::from("href"), Value::String(link.to_string())),
        (String::from("title"), Value::String(title.to_string())),
      ])),
      Image(_, link, title) => Some(HashMap::from([
        (String::from("src"), Value::String(link.to_string())),
        (String::from("title"), Value::String(title.to_string())),
      ])),
      _ => None,
    },
    Text(text) | Code(text) => Some(HashMap::from([(
      String::from("content"),
      Value::String(text.to_string()),
    )])),
    _ => None,
  }
}
