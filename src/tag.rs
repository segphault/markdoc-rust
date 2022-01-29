use crate::model::{Attributes, Value};
use pest::error::Error;
use pest::iterators::Pair;
use pest::{Parser, RuleType};

#[derive(Parser)]
#[grammar = "grammar/tag.pest"]
struct TagParser;

#[derive(PartialEq, Debug)]
pub enum Tag<'a, R: RuleType> {
  Standalone(&'a str, Option<Attributes>),
  Open(&'a str, Option<Attributes>),
  Close(&'a str),
  Annotation(Attributes),
  Value(Value),
  Error(Error<R>),
}

impl<'a> From<&'a str> for Tag<'a, Rule> {
  fn from(input: &'a str) -> Tag<'a, Rule> {
    match TagParser::parse(Rule::Top, input) {
      Ok(mut pair) => convert_tag(pair.next().unwrap()),
      Err(err) => Tag::Error(err),
    }
  }
}

fn convert_tag(pair: Pair<Rule>) -> Tag<Rule> {
  match pair.as_rule() {
    Rule::Function | Rule::Variable => Tag::Value(convert_value(pair)),
    Rule::Annotation => Tag::Annotation(convert_attributes(pair.into_inner().next().unwrap())),
    Rule::TagClose => Tag::Close(pair.into_inner().next().unwrap().as_str()),
    Rule::TagOpen => {
      let standalone = pair.as_str().trim().ends_with('/');
      let mut inner = pair.into_inner();
      let name = inner.next().unwrap().as_str();
      let mut attrs = Attributes::new();

      for item in inner {
        match item.as_rule() {
          Rule::Primary => {
            attrs.insert(
              String::from("primary"),
              convert_value(item.into_inner().next().unwrap()),
            );
          }
          Rule::Attributes => {
            attrs.extend(convert_attributes(item));
          }
          _ => (),
        }
      }

      let attributes = if attrs.is_empty() { None } else { Some(attrs) };

      if standalone {
        Tag::Standalone(name, attributes)
      } else {
        Tag::Open(name, attributes)
      }
    }
    _ => unreachable!(),
  }
}

fn convert_function(pair: Pair<Rule>) -> Value {
  let mut inner = pair.into_inner();
  let name = inner.next().unwrap().as_str();
  let attrs: Attributes = inner
    .enumerate()
    .map(|(index, item)| match item.as_rule() {
      Rule::Attribute => {
        let mut inner = item.into_inner();
        let key = inner.next().unwrap().as_str();
        let value = convert_value(inner.next().unwrap());
        (String::from(key), value)
      }
      _ => (index.to_string(), convert_value(item)),
    })
    .collect();

  Value::Function(name.to_string(), attrs)
}

fn convert_attributes(pair: Pair<Rule>) -> Attributes {
  let mut classes = Attributes::new();
  let mut attributes: Attributes = pair
    .into_inner()
    .filter_map(|item| {
      let mut inner = item.into_inner();
      let key = inner.next().unwrap().as_str();
      let value = inner.next().unwrap();

      match key {
        "." | "class" => {
          classes.insert(String::from(value.as_str()), Value::Boolean(true));
          None
        }
        "#" => Some((String::from("id"), convert_value(value))),
        _ => Some((String::from(key), convert_value(value))),
      }
    })
    .collect();

  if !classes.is_empty() {
    attributes.insert(String::from("class"), Value::Hash(classes));
  }

  attributes
}

fn convert_value(pair: Pair<Rule>) -> Value {
  match pair.as_rule() {
    Rule::Variable => Value::Variable(
      pair.as_str().chars().nth(0).unwrap(),
      pair.into_inner().map(convert_value).collect(),
    ),
    Rule::Function => convert_function(pair),
    Rule::ValueNull => Value::Null,
    Rule::Identifier | Rule::ValueString => Value::String(String::from(pair.as_str())),
    Rule::ValueNumber => Value::Number(pair.as_str().parse().unwrap()),
    Rule::ValueBoolean => Value::Boolean(pair.as_str().parse().unwrap()),
    Rule::ValueArray => Value::Array(pair.into_inner().map(convert_value).collect()),
    Rule::ValueHash => Value::Hash(
      pair
        .into_inner()
        .map(|item| {
          let mut inner = item.into_inner();
          let key = inner.next().unwrap().as_str();
          let value = convert_value(inner.next().unwrap());
          (String::from(key), value)
        })
        .collect(),
    ),
    _ => unreachable!(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashMap;

  #[test]
  fn parse_basic_tag_open() {
    let output = Tag::from("{% foo %}");
    assert_eq!(output, Tag::Open("foo", None))
  }

  #[test]
  fn parse_tag_open_with_attributes() {
    let output = Tag::from("{% foo bar=1 baz=true %}");
    assert_eq!(
      output,
      Tag::Open(
        "foo",
        Some(HashMap::from([
          (String::from("bar"), Value::Number(1.0)),
          (String::from("baz"), Value::Boolean(true))
        ]))
      )
    )
  }

  #[test]
  fn parse_tag_open_with_primary() {
    let output = Tag::from("{% foo $bar %}");
    assert_eq!(
      output,
      Tag::Open(
        "foo",
        Some(HashMap::from([(
          String::from("primary"),
          Value::Variable('$', vec![Value::String(String::from("bar"))])
        )]))
      )
    )
  }

  #[test]
  fn parse_tag_open_with_primary_and_attributes() {
    let output = Tag::from("{% foo $bar baz=true %}");
    assert_eq!(
      output,
      Tag::Open(
        "foo",
        Some(HashMap::from([
          (
            String::from("primary"),
            Value::Variable('$', vec![Value::String(String::from("bar"))])
          ),
          (String::from("baz"), Value::Boolean(true))
        ]))
      )
    )
  }

  #[test]
  fn parse_annotation_with_attributes() {
    let output = Tag::from("{% foo={bar: 1} baz=\"test\" %}");
    assert_eq!(
      output,
      Tag::Annotation(HashMap::from([
        (
          String::from("foo"),
          Value::Hash(HashMap::from([(String::from("bar"), Value::Number(1.0))]))
        ),
        (String::from("baz"), Value::String(String::from("test")))
      ]))
    )
  }

  #[test]
  fn parse_self_closing_tag() {
    let output = Tag::from("{% foo bar=\"baz\" /%}");
    assert_eq!(
      output,
      Tag::Standalone(
        "foo",
        Some(HashMap::from([(
          String::from("bar"),
          Value::String(String::from("baz"))
        )]))
      )
    )
  }

  #[test]
  fn convert_value_array() {
    let pair = TagParser::parse(Rule::Value, "[1, 2, 3]")
      .expect("parse failed")
      .next()
      .unwrap();

    assert_eq!(
      convert_value(pair),
      Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0)
      ])
    );
  }

  #[test]
  fn convert_value_array_with_trailing_comma() {
    let pair = TagParser::parse(Rule::Value, "[1, 2, 3, ]")
      .expect("parse failed")
      .next()
      .unwrap();

    assert_eq!(
      convert_value(pair),
      Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0)
      ])
    );
  }

  #[test]
  fn convert_value_hash() {
    let pair = TagParser::parse(Rule::Value, "{foo: \"bar\", baz: true}")
      .expect("parse failed")
      .next()
      .unwrap();

    assert_eq!(
      convert_value(pair),
      Value::Hash(HashMap::from([
        (String::from("foo"), Value::String(String::from("bar"))),
        (String::from("baz"), Value::Boolean(true))
      ]))
    )
  }

  #[test]
  fn convert_parsed_attributes() {
    let pair = TagParser::parse(Rule::Attributes, "foo=true bar=[1, 2, 3]")
      .expect("parse failed")
      .next()
      .unwrap();

    assert_eq!(
      convert_attributes(pair),
      HashMap::from([
        (String::from("foo"), Value::Boolean(true)),
        (
          String::from("bar"),
          Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0)
          ])
        )
      ])
    )
  }

  #[test]
  fn convert_empty_string() {
    let pair = TagParser::parse(Rule::Attributes, "foo=true bar=\"\"")
      .expect("parse failed")
      .next()
      .unwrap();

    assert_eq!(
      convert_attributes(pair),
      Attributes::from([
        ("foo".to_string(), Value::Boolean(true)),
        ("bar".to_string(), Value::String("".to_string()))
      ])
    )
  }

  #[test]
  fn convert_attribute_shortcuts() {
    let pair = TagParser::parse(Rule::Attributes, "asdf=1 #foo .bar .baz")
      .expect("parse failed")
      .next()
      .unwrap();

    assert_eq!(
      convert_attributes(pair),
      HashMap::from([
        (String::from("asdf"), Value::Number(1.0)),
        (String::from("id"), Value::String(String::from("foo"))),
        (
          String::from("class"),
          Value::Hash(Attributes::from([
            (String::from("bar"), Value::Boolean(true)),
            (String::from("baz"), Value::Boolean(true)),
          ]))
        )
      ])
    )
  }

  #[test]
  fn convert_variable() {
    let pair = TagParser::parse(Rule::Variable, "$foo.bar[10].baz")
      .expect("parse failed")
      .next()
      .unwrap();

    assert_eq!(
      convert_value(pair),
      Value::Variable(
        '$',
        vec![
          Value::String(String::from("foo")),
          Value::String(String::from("bar")),
          Value::Number(10.0),
          Value::String(String::from("baz")),
        ]
      )
    )
  }

  #[test]
  fn converting_function() {
    let pair = TagParser::parse(Rule::Function, "foo(1, bar=true, 3)")
      .expect("parse failed")
      .next()
      .unwrap();

    assert_eq!(
      convert_function(pair),
      Value::Function(
        "foo".to_string(),
        Attributes::from([
          ("0".to_string(), Value::Number(1.0)),
          ("bar".to_string(), Value::Boolean(true)),
          ("2".to_string(), Value::Number(3.0)),
        ])
      )
    )
  }

  #[test]
  fn tag_with_function_attribute_and_variable() {
    let tag = Tag::from("{% foo bar=baz($test) %}");
    assert_eq!(
      tag,
      Tag::Open(
        "foo",
        Some(Attributes::from([(
          "bar".to_string(),
          Value::Function(
            "baz".to_string(),
            Attributes::from([(
              "0".to_string(),
              Value::Variable('$', vec![Value::String("test".to_string())])
            )])
          )
        )]))
      )
    )
  }
}

// badges=[\"Client-side\" ]
// (
//   "badges".to_string(),
//   Value::Array(vec![Value::String("Client-side".to_string())])
// )
