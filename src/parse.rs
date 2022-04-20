use crate::model::*;
use crate::tokenize::{tokenize, Events};
use crate::{tag, template};
use pulldown_cmark::{scan_markdoc_tag_end, CodeBlockKind, Event, Tag as EventTag};
use std::ops::Deref;
use tag::Tag;

pub fn extract_frontmatter(input: &str) -> Option<(&str, &str)> {
  input
    .strip_prefix("---")
    .and_then(|left| left.split_once("---"))
}

pub fn parse(input: &str) -> NodeRef {
  let mut attributes = None;
  let mut offset = 0;
  let mut source = input;

  if let Some((frontmatter, text)) = extract_frontmatter(input) {
    attributes = Some(mdattrs!(frontmatter = frontmatter.trim()));
    offset = input.len() - text.len();
    source = text;
  }

  let root = NodeRef::from(Node {
    kind: Type::Document,
    location: Some(0..input.len()),
    attributes,
    ..Node::default()
  });

  let events = tokenize(source);
  let mut nodes = vec![root.clone()];
  convert_events(source, &mut nodes, events.collect(), offset, true);
  root
}

fn add_child<'a>(nodes: &mut Vec<NodeRef<'a>>, child: NodeRef<'a>, push: bool) {
  if let Some(last) = nodes.last() {
    last.borrow_mut().push(child.clone());
  }

  if push {
    nodes.push(child)
  }
}

fn parent_kind(nodes: &[NodeRef], kind: Type) -> bool {
  nodes
    .last()
    .map(|x| x.borrow().kind == kind)
    .unwrap_or(false)
}

fn event_type(event: &Event) -> Type {
  match event {
    Event::End(tag) | Event::Start(tag) => match tag {
      EventTag::Paragraph => Type::Paragraph,
      EventTag::Heading(..) => Type::Heading,
      EventTag::BlockQuote => Type::Blockquote,
      EventTag::CodeBlock(..) => Type::Fence,
      EventTag::List(..) => Type::List,
      EventTag::Item => Type::Item,
      EventTag::Table(..) => Type::Table,
      EventTag::TableHead => Type::TableHead,
      EventTag::TableRow => Type::TableRow,
      EventTag::TableCell => Type::TableCell,
      EventTag::Emphasis => Type::Emphasis,
      EventTag::Strong => Type::Strong,
      EventTag::Strikethrough => Type::Strike,
      EventTag::Link(..) => Type::Link,
      EventTag::Image(..) => Type::Image,
      _ => Type::Nop,
    },
    Event::Text(..) => Type::Text,
    Event::Code(..) => Type::Code,
    Event::SoftBreak => Type::SoftBreak,
    Event::HardBreak => Type::HardBreak,
    Event::Rule => Type::Rule,
    Event::MarkdocTag(_, inline) => Type::Tag(*inline),
    _ => Type::Nop,
  }
}

pub fn convert_events<'a>(
  input: &'a str,
  nodes: &mut Vec<NodeRef<'a>>,
  events: Events<'a>,
  offset: usize,
  add_inlines: bool,
) {
  let mut last_inline: Option<NodeRef> = None;
  let mut inside_thead = false;
  let mut inside_fence = false;

  for (event, range) in events {
    let kind = event_type(&event);
    let offset_range = range.start + offset..range.end + offset;

    if inside_fence {
      if let Event::End(EventTag::CodeBlock(..)) = &event {
        inside_fence = false;
        nodes.pop();
      }

      continue;
    }

    if add_inlines {
      if last_inline.is_none() && kind.is_inline() {
        let inline_node = mdnode!(Type::Inline, None);
        if let Some(parent) = nodes.last_mut() {
          last_inline = Some(parent.clone());
        }

        add_child(nodes, inline_node, true);
      }

      if last_inline.is_some() && !kind.is_inline() && parent_kind(nodes, Type::Inline) {
        last_inline = None;
        nodes.pop();
      }
    }

    match event {
      Event::Text(text) | Event::Code(text) => {
        add_child(nodes, mdnode!(kind, offset_range, content = text), false);
      }

      Event::SoftBreak | Event::HardBreak => {
        add_child(nodes, mdnode!(kind, offset_range), false);
      }

      Event::MarkdocTag(_, inline) => {
        let tag = tag::parse(&input[range.clone()]);
        let push = matches!(&tag, Tag::Open(..));

        match tag {
          Tag::Open(name, attributes) | Tag::Standalone(name, attributes) => {
            let node = mdnode!(Type::Tag(inline), offset_range, attributes);
            node.borrow_mut().tag = Some(name.into());
            add_child(nodes, node, push);
          }

          Tag::Close(name) => {
            if nodes
              .last()
              .and_then(|x| x.borrow().tag.as_ref().map(|x| x.deref() == name))
              .unwrap_or(false)
            {
              nodes.pop();
              continue;
            }

            let error = Error {
              id: "missing-opening",
              level: ErrorLevel::Critical,
              message: format!("Tag '{}' is missing opening", name),
              location: Some(offset_range.clone()),
            };

            let node = Node {
              kind: Type::Tag(inline),
              tag: Some(name.into()),
              location: Some(offset_range),
              errors: vec![error].into(),
              ..Node::default()
            };

            add_child(nodes, node.into(), false);
          }

          Tag::Annotation(attributes) => {
            if let Some(node) = &last_inline {
              node.borrow_mut().set_attributes(attributes);
            }
          }

          Tag::Value(variable) => {
            let node = mdnode!(Type::Text, offset_range, content = variable);
            add_child(nodes, node, false);
          }

          Tag::Error(error) => {
            let err = Error {
              id: "syntax-error",
              level: ErrorLevel::Critical,
              message: format!("{}", error),
              location: Some(offset_range.clone()),
            };

            let node = Node {
              kind: Type::Error,
              location: Some(offset_range),
              errors: vec![err].into(),
              ..Node::default()
            };

            add_child(nodes, node.into(), false);
          }
        };
      }

      Event::Start(tag) => match tag {
        EventTag::Heading(level, ..) => {
          let node = mdnode!(kind, offset_range, level = level as i32);
          add_child(nodes, node, true);
        }

        EventTag::List(ordered) => {
          let attrs = match ordered {
            Some(n) => mdattrs!(ordered = true, number = n as i32),
            None => mdattrs!(ordered = false),
          };

          add_child(nodes, mdnode!(kind, offset_range, attrs), true);
        }

        EventTag::Link(_, link, title) => {
          let attrs = if title.is_empty() {
            mdattrs!(href = link)
          } else {
            mdattrs!(href = link, title = title)
          };

          add_child(nodes, mdnode!(kind, offset_range, attrs), true);
        }

        EventTag::Image(_, link, title) => {
          let attrs = if title.is_empty() {
            mdattrs!(src = link)
          } else {
            mdattrs!(src = link, title = title)
          };

          add_child(nodes, mdnode!(kind, offset_range, attrs), true);
        }

        EventTag::CodeBlock(CodeBlockKind::Fenced(info)) => {
          let mut content = &input[range.clone()];
          let mut attributes = Attributes::new();

          if let (Some(start), Some(end)) = (content.find('\n'), content.rfind('\n')) {
            if let Some(lang) = info.split_ascii_whitespace().next() {
              attributes.insert("language".into(), Value::from(lang.to_owned()));
            }

            if let Some(tag_start) = content[0..start].find("{%") {
              if let Some(end) = scan_markdoc_tag_end(content[tag_start..start].as_bytes()) {
                if let Tag::Annotation(attrs) = tag::parse(&content[tag_start..(tag_start + end)]) {
                  attributes.extend(attrs);
                }
              }
            }

            content = &content[start..=end];
            inside_fence = true;
            attributes.insert("content".into(), Value::from(content));

            let node = mdnode!(kind, offset_range.clone(), attributes);
            let events = template::parse(content);
            add_child(nodes, node.clone(), true);
            convert_events(content, nodes, events, offset + range.start + start, false);
          };
        }

        EventTag::TableHead => {
          inside_thead = true;
          add_child(nodes, mdnode!(Type::TableHead, offset_range.clone()), true);
          add_child(nodes, mdnode!(Type::TableRow, offset_range), true);
        }

        EventTag::TableCell => {
          let kind = if inside_thead {
            Type::TableHeadCell
          } else {
            Type::TableCell
          };

          add_child(nodes, mdnode!(kind, offset_range), true);
        }

        _ => {
          add_child(nodes, mdnode!(kind, offset_range), true);
        }
      },

      Event::End(tag) => {
        if matches!(tag, EventTag::Table(..)) && parent_kind(nodes, Type::TableBody) {
          nodes.pop();
        }

        nodes.pop();

        if matches!(tag, EventTag::TableHead) {
          inside_thead = false;
          let node = mdnode!(Type::TableBody, None);
          add_child(nodes, node.clone(), true);
        }
      }

      Event::Rule => add_child(nodes, mdnode!(Type::Rule, range), false),

      _ => (),
    }
  }
}
