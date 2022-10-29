use crate::model::render::Renderable;
use std::io::{Error, Write};

static VOID_ELEMENTS: &[&str] = &[
  "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
  "track", "wbr",
];

pub fn render<W: Write>(node: &Renderable, writer: &mut W) -> Result<(), Error> {
  match node {
    Renderable::String(value) => {
      writer.write(value.as_bytes())?;
    }

    Renderable::Fragment(children) => {
      for child in children {
        render(&child, writer)?;
      }
    }

    Renderable::Tag {
      name,
      attributes,
      children,
    } => {
      write!(writer, "<{}", name)?;

      if let Some(attrs) = attributes {
        for (key, value) in attrs {
          write!(writer, " {}={}", key, value)?;
        }
      }

      write!(writer, ">")?;

      if VOID_ELEMENTS.contains(&name.as_ref()) {
        return Ok(());
      }

      if let Some(children) = children {
        for child in children {
          render(&child, writer)?;
        }
      }

      write!(writer, "</{}>", name)?;
    }

    _ => (),
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use std::io::BufWriter;

  use crate::{
    model::schema::Config, parse::parse, schema::default_nodes, transform::transform_node,
  };

  use super::*;

  #[test]
  fn example() {
    let config = Config {
      tags: hash!(),
      nodes: default_nodes(),
    };

    let doc = parse(
      r#"
# This is a test

This is a sample document

* This is a bulleted list
* With another list item
"#,
    );

    let parent = doc.borrow();
    let rendered = transform_node(&*parent, &config);
    let mut writer = BufWriter::new(Vec::new());

    dbg!(render(&rendered, &mut writer));

    if let Ok(value) = std::str::from_utf8(writer.buffer()) {
      dbg!(value);
    }
  }
}
