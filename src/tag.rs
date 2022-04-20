use pulldown_cmark::CowStr;
use crate::model::{Attributes, Value};
use pest::error::Error;
use pest::iterators::Pair;
use pest::{Parser, RuleType};

#[derive(Parser)]
#[grammar = "tag.pest"]
struct TagParser;

#[derive(PartialEq, Debug)]
pub enum Tag<'a, R: RuleType> {
  Standalone(&'a str, Option<Attributes<'a>>),
  Open(&'a str, Option<Attributes<'a>>),
  Close(&'a str),
  Annotation(Attributes<'a>),
  Value(Value<'a>),
  Error(Error<R>),
}

pub fn parse(input: &str) -> Tag<Rule> {
  match TagParser::parse(Rule::Top, input) {
    Ok(mut pair) => convert_tag(pair.next().unwrap()),
    Err(err) => Tag::Error(err)
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
              "primary".into(),
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
        (key.into(), value)
      }
      _ => (CowStr::from(index.to_string()), convert_value(item)),
    })
    .collect();

  Value::Function(name.into(), attrs)
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
          classes.insert(value.as_str().into(), Value::Boolean(true));
          None
        }
        "#" => Some(("id".into(), convert_value(value))),
        _ => Some((key.into(), convert_value(value))),
      }
    })
    .collect();

  if !classes.is_empty() {
    attributes.insert("class".into(), Value::Hash(classes));
  }

  attributes
}

fn convert_value(pair: Pair<Rule>) -> Value {
  match pair.as_rule() {
    Rule::Variable => Value::Variable(
      pair.as_str().chars().next().unwrap(),
      pair.into_inner().map(convert_value).collect(),
    ),
    Rule::Function => convert_function(pair),
    Rule::ValueNull => Value::Null,
    Rule::Identifier | Rule::ValueString => pair.as_str().into(),
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
          (key.into(), value)
        })
        .collect(),
    ),
    _ => unreachable!(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_basic_tag_open() {
    let output = parse("{% foo %}");
    assert_eq!(output, Tag::Open("foo", None))
  }

  #[test]
  fn parse_tag_open_with_attributes() {
    let output = parse("{% foo bar=1 baz=true %}");
    assert_eq!(
      output,
      Tag::Open(
        "foo",
        Some([("bar".into(), 1.into()), ("baz".into(), true.into())].into())
      )
    )
  }

  #[test]
  fn parse_tag_open_with_primary() {
    let output = parse("{% foo $bar %}");
    assert_eq!(
      output,
      Tag::Open(
        "foo",
        Some([("primary".into(), Value::Variable('$', vec!["bar".into()]))].into())
      )
    )
  }

  #[test]
  fn parse_tag_open_with_primary_and_attributes() {
    let output = parse("{% foo $bar baz=true %}");
    assert_eq!(
      output,
      Tag::Open(
        "foo",
        Some(
          [
            ("primary".into(), Value::Variable('$', vec!["bar".into()])),
            ("baz".into(), true.into())
          ]
          .into()
        )
      )
    )
  }

  #[test]
  fn parse_annotation_with_attributes() {
    let output = parse("{% foo={bar: 1} baz=\"test\" %}");
    assert_eq!(
      output,
      Tag::Annotation(
        [
          ("foo".into(), Value::Hash([("bar".into(), 1.into())].into())),
          ("baz".into(), "test".into())
        ]
        .into()
      )
    )
  }

  #[test]
  fn parse_self_closing_tag() {
    let output = parse("{% foo bar=\"baz\" /%}");
    assert_eq!(
      output,
      Tag::Standalone("foo", Some([("bar".into(), "baz".into())].into()))
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
      vec![1.into(), 2.into(), 3.into()].into()
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
      vec![1.into(), 2.into(), 3.into()].into()
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
      mdvalue!({"foo": "bar", "baz": true})
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
      [
        ("foo".into(), true.into()),
        ("bar".into(), vec![1.into(), 2.into(), 3.into()].into())
      ]
      .into()
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
      [("foo".into(), true.into()), ("bar".into(), "".into())].into()
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
      mdattrs!(asdf=1, id="foo", class={"bar": true, "baz": true})
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
        vec!["foo".into(), "bar".into(), 10.into(), "baz".into()]
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
        "foo".into(),
        [
          ("0".into(), 1.into()),
          ("bar".into(), true.into()),
          ("2".into(), 3.into()),
        ]
        .into()
      )
    )
  }

  #[test]
  fn tag_with_function_attribute_and_variable() {
    let tag = parse("{% foo bar=baz($test) %}");
    assert_eq!(
      tag,
      Tag::Open(
        "foo",
        Some(
          [(
            "bar".into(),
            Value::Function(
              "baz".into(),
              [("0".into(), Value::Variable('$', vec!["test".into()]))].into()
            )
          )]
          .into()
        )
      )
    )
  }
}
