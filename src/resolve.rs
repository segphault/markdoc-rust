use crate::model::{
  schema::{Config, Variables},
  value::{Expression, Value},
  Attributes, node::Node,
};

pub fn resolve_attributes<'a>(attributes: &Attributes<'a>, config: &'a Config<'a>) {
  for (_, value) in attributes {
    resolve(value, config)
  }
}

pub fn resolve<'a>(value: &Value<'a>, config: &'a Config<'a>) {
  match value {
    Value::Hash(attributes) => resolve_attributes(attributes, config),

    Value::Array(values) => {
      for value in values {
        resolve(value, config)
      }
    }

    Value::Expression(value, Expression::Function(name, parameters)) => {
      *value.borrow_mut() = config
        .functions
        .as_ref()
        .and_then(|fns| fns.get(name.as_ref()))
        .map_or(Value::Undefined, |f| {
          resolve_attributes(parameters, config);
          (f.evaluate)(parameters, config)
        })
    }

    Value::Expression(value, Expression::Variable(_, path)) => {
      *value.borrow_mut() = match &config.variables {
        Some(Variables::Resolver(vfn)) => Some(vfn(&path[..])),
        Some(Variables::Values(variables)) => match &path[..] {
          [Value::String(first)] => variables.get(first).map(|x| x.clone()),
          [Value::String(first), rest @ ..] => variables
            .get(first)
            .and_then(|vars| vars.deep_get(rest))
            .map(|x| x.clone()),
          _ => None,
        },
        _ => None,
      }
      .unwrap_or(Value::Undefined)
    }

    _ => (),
  }
}

pub fn resolve_node<'a>(node: &Node<'a>, config: &'a Config<'a>) {
  if let Some(attributes) = &node.attributes {
    resolve_attributes(attributes, config);
  }

  if let Some(children) = &node.children {
    for child in children {
      resolve_node(&child.borrow(), config)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::model::schema::FunctionSchema;
  use crate::model::value::Expression;

  #[test]
  fn example() -> Result<(), std::io::Error> {
    let config = Config {
      tags: hash!(),
      nodes: hash!(),
      variables: Variables::Values(hash!(
        "foo".into() => hash!(
          "bar".into() => "variable resolved".into()
        ).into()
      ))
      .into(),
      functions: hash!(
        "foo" => FunctionSchema {
          attributes: None,
          evaluate: |_attrs, _config| "function resolved".into(),
        }
      )
      .into(),
    };

    let array = Value::Array(vec![
      "test".into(),
      Expression::Variable('$', vec!["foo".into(), "bar".into()]).into(),
      Expression::Function("foo".into(), mdattrs!(example = "test")).into(),
    ]);

    resolve(&array, &config);

    match &array {
      Value::Array(values) => match &values.as_slice() {
        [_, Value::Expression(second, _), Value::Expression(third, _)] => {
          assert_eq!(*second.borrow(), Value::String("variable resolved".into()));
          assert_eq!(*third.borrow(), Value::String("function resolved".into()));
          Ok(())
        }
        _ => Err(std::io::Error::new(
          std::io::ErrorKind::Other,
          "wrong number of list items",
        )),
      },
      _ => Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "not an array",
      )),
    }
  }
}
