use crate::model::node::*;
use crate::model::render::*;
use crate::model::schema::*;
use crate::model::value::*;
use crate::transform::*;
use std::collections::HashMap;

pub fn default_nodes<'a>() -> HashMap<Type, Schema<'a>> {
  hash!(
    Type::Document => Schema {
      attributes: hash!(
        "frontmatter" => Attribute {
          render: AttributeRender::False,
          ..Default::default()
        }
      ).into(),
      ..Default::default()
    },

    Type::Heading => Schema {
      attributes: hash!(
        "level" => Attribute {
          kind: Some(SchemaType::String),
          render: AttributeRender::False,
          required: true,
        }
      ).into(),
      transform: Transform::Function(|node, config| Renderable::Tag {
        name: format!("h{}", node.attribute("level").unwrap()).into(),
        attributes: transform_attributes(&node, &config),
        children: transform_children(&node, &config)
      }).into(),
      ..Default::default()
    },

    Type::Paragraph => Schema {
      render: "p".into(),
      ..Default::default()
    },

    Type::Image => Schema {
      render: "img".into(),
      attributes: hash!(
        "src" => Attribute {
          kind: Some(SchemaType::String),
          required: true,
          ..Default::default()
        },
        "alt" => Attribute {
          kind: Some(SchemaType::String),
          ..Default::default()
        },
        "title" => Attribute {
          kind: Some(SchemaType::String),
          ..Default::default()
        }
      ).into(),
      ..Default::default()
    },

    Type::Fence => Schema {
      render: "pre".into(),
      attributes: hash!(
        "content" => Attribute {
          kind: Some(SchemaType::String),
          render: AttributeRender::False,
          required: true,
        },
        "language" => Attribute {
          kind: Some(SchemaType::String),
          render: AttributeRender::Name("data-language"),
          ..Default::default()
        },
        "process" => Attribute {
          kind: Some(SchemaType::Boolean),
          render: AttributeRender::False,
          ..Default::default()
        }
      ).into(),
      ..Default::default()
    },

    Type::Blockquote => Schema {
      render: "blockquote".into(),
      ..Default::default()
    },

    Type::Item => Schema {
      render: "li".into(),
      ..Default::default()
    },

    Type::List => Schema {
      attributes: hash!(
        "ordered" => Attribute {
          kind: Some(SchemaType::Boolean),
          render: AttributeRender::False,
          required: true,
          ..Default::default()
        }
      ).into(),
      transform: Transform::Function(|node, config| Renderable::Tag {
        name: match node.attribute("ordered") {
          Some(Value::Boolean(true)) => "ol",
          _ => "ul"
        }.into(),
        attributes: transform_attributes(&node, &config),
        children: transform_children(&node, &config)
      }).into(),
      ..Default::default()
    },

    Type::Strong => Schema {
      render: "strong".into(),
      ..Default::default()
    },

    Type::Emphasis => Schema {
      render: "em".into(),
      ..Default::default()
    },

    Type::Strike => Schema {
      render: "s".into(),
      ..Default::default()
    },

    Type::Code => Schema {
      render: "code".into(),
      attributes: hash!(
        "content" => Attribute {
          kind: Some(SchemaType::String),
          render: AttributeRender::False,
          required: true,
          ..Default::default()
        }
      ).into(),
      transform: Transform::Function(|node, config| Renderable::Tag {
        name: "code".into(),
        attributes: transform_attributes(&node, &config),
        children: vec![
          Renderable::String("asdf".into()).into()
        ].into()
      }).into(),
      ..Default::default()
    },

    Type::Link => Schema {
      render: "a".into(),
      attributes: hash!(
        "href" => Attribute {
          kind: Some(SchemaType::String),
          required: true,
          ..Default::default()
        },
        "title" => Attribute {
          kind: Some(SchemaType::String),
          ..Default::default()
        }
      ).into(),
      ..Default::default()
    },

    Type::Inline => Schema {
      ..Default::default()
    },

    Type::Text => Schema {
      attributes: hash!(
        "content" => Attribute {
          kind: Some(SchemaType::String),
          required: true,
          ..Default::default()
        }
      ).into(),
      transform: Transform::Function(|node, _config| {
        if let Some(Value::String(value)) = node.attribute("content") {
          Renderable::String(value.clone())
        } else {
          Renderable::Null
        }
      }).into(),
      ..Default::default()
    },

    Type::HardBreak => Schema {
      render: "br".into(),
      ..Default::default()
    },

    Type::SoftBreak => Schema {
      transform: Transform::Function(|_node, _config|
        Renderable::String(" ".into())
      ).into(),
      ..Default::default()
    }
  )
}
