use crate::model::{node::*, render::*, schema::*, Attributes};

pub fn find_schema<'a>(node: &Node<'a>, config: &'a Config<'a>) -> Option<&'a Schema<'a>> {
  node
    .tag
    .as_ref()
    .and_then(|tag| config.tags.get(tag.as_ref()))
    .or_else(|| config.nodes.get(&node.kind))
}

pub fn transform_attributes<'a>(node: &Node<'a>, config: &'a Config<'a>) -> Option<Attributes<'a>> {
  if let Some(attrs) = find_schema(&node, &config).and_then(|schema| schema.attributes.as_ref()) {
    let mut output = Attributes::new();

    for (key, attr) in attrs {
      let name = match attr.render {
        AttributeRender::True => key,
        AttributeRender::Name(n) => n,
        AttributeRender::False => continue,
      };

      if let Some(value) = node.attribute(key) {
        output.insert(name.into(), value.clone());
      }
    }

    if !output.is_empty() {
      return output.into();
    }
  }

  None
}

pub fn transform_children<'a>(
  node: &Node<'a>,
  config: &'a Config<'a>,
) -> Option<Vec<Renderable<'a>>> {
  node.children.as_ref().and_then(|children| {
    Some(
      children
        .iter()
        .map(|ch| transform_node(&ch.borrow(), &config).into())
        .collect(),
    )
  })
}

pub fn transform_node<'a>(node: &Node<'a>, config: &'a Config<'a>) -> Renderable<'a> {
  if let Some(schema) = find_schema(&node, &config) {
    if let Some(transform_func) = schema.transform {
      return transform_func(&node, &config);
    }

    let children = transform_children(&node, &config);

    if let Some(render) = schema.render {
      return Renderable::Tag {
        name: render.into(),
        attributes: transform_attributes(&node, &config),
        children,
      };
    }

    if let Some(children) = children {
      return Renderable::Fragment(children);
    }
  }

  Renderable::Null
}
