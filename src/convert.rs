use crate::markdown;
use crate::model::{Attributes, Token, Type};
use crate::tag::{Rule, Tag};

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
      MarkdocTag(text, inline) => convert_tag(Tag::from(text.as_ref()), inline),
      _ => Token::Append {
        kind,
        attributes: convert_attributes(event),
      },
    }
  }
}

fn convert_fenced_info(info: &str) -> Option<Attributes> {
  let language = info.split_ascii_whitespace().next().unwrap_or("").into();

  if let Some(start) = info.find("{%") {
    if let Some(end) = markdown::scan_markdoc_tag_end(info[start..].as_bytes()) {
      if let Tag::Annotation(mut attrs) = Tag::from(&info[start..(start + end)]) {
        attrs.insert("language".into(), language);
        return Some(attrs);
      }
    }
  }

  Some([("language".into(), language)].into())
}

pub(crate) fn convert_tag(tag: Tag<Rule>, inline: bool) -> Token {
  match tag {
    Tag::Standalone(name, attributes) => Token::Append {
      kind: Type::Tag {
        name: name.into(),
        inline,
      },
      attributes,
    },
    Tag::Open(name, attributes) => Token::Open {
      kind: Type::Tag {
        name: name.into(),
        inline,
      },
      attributes,
    },
    Tag::Close(name) => Token::Close {
      kind: Type::Tag {
        name: name.into(),
        inline,
      },
    },
    Tag::Value(variable) => Token::Append {
      kind: Type::Text,
      attributes: Some([("content".into(), variable)].into()),
    },
    Tag::Annotation(attributes) => Token::Annotate {
      attributes: Some(attributes),
    },
    Tag::Error(error) => Token::Error {
      message: error.to_string(),
    },
  }
}

pub(crate) fn convert_attributes(event: markdown::Event) -> Option<Attributes> {
  use markdown::CodeBlockKind::*;
  use markdown::Event::*;
  use markdown::Tag::*;

  match event {
    End(tag) | Start(tag) => match tag {
      Heading(level, ..) => Some([("level".into(), (level as u8).into())].into()),
      CodeBlock(kind) => match kind {
        Fenced(info) => convert_fenced_info(&info.to_string()),
        Indented => None,
      },
      List(ordered) => match ordered {
        Some(n) => Some([("ordered".into(), true.into()), ("number".into(), n.into())].into()),
        None => Some([("ordered".into(), false.into())].into()),
      },
      Link(_, link, title) => Some(if title.is_empty() {
        [("href".into(), link.into())].into()
      } else {
        [("href".into(), link.into()), ("title".into(), title.into())].into()
      }),
      Image(_, link, title) => Some(if title.is_empty() {
        [("src".into(), link.into())].into()
      } else {
        [("src".into(), link.into()), ("title".into(), title.into())].into()
      }),
      _ => None,
    },
    Text(text) | Code(text) => Some([("content".into(), text.into())].into()),
    _ => None,
  }
}
