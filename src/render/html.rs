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
          write!(writer, r#" {}="{}""#, key, value)?;
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
  use super::*;
  use crate::{
    model::schema::{Config, Variables},
    parse::parse,
    schema::default_nodes,
    transform::transform_node,
  };
  use std::io::BufWriter;

  #[test]
  fn example() {
    let schema = r#"
{
  "nodes": {},
  "tags": {
    "foo": {
      "render": "foo",
      "attributes": {
        "bar": {"kind": "String"}
      }
    }
  }
}
"#;

    let doc = parse(
      r#"
# This is a test

---

This is a sample document

* This is a bulleted list
* With another list item

{% foo bar="test" %}
This is a test: {% $foo.bar %}
{% /foo %}
"#,
    );

    let config = serde_json::from_str::<Config>(&schema).unwrap();

    let config = Config {
      tags: config.tags,
      nodes: default_nodes(),
      variables: Variables::Values(hash!(
        "foo".into() => hash!(
          "bar".into() => "variable resolved".into()
        ).into()
      ))
      .into(),
      functions: None,
    };

    let parent = doc.borrow();
    crate::resolve::resolve_node(&parent, &config);
    let rendered = transform_node(&*parent, &config);
    let mut writer = BufWriter::new(Vec::new());
    render(&rendered, &mut writer).expect("completes");
    let output = std::str::from_utf8(writer.buffer());
    let expected = r#"<h1>This is a test</h1><hr><p>This is a sample document</p><ul><li>This is a bulleted list</li><li>With another list item</li></ul><foo bar="test"><p>This is a test: variable resolved</p></foo>"#;
    assert_eq!(Ok(expected), output);
  }
}
