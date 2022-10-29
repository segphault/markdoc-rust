
macro_rules! hash {
  ($($k:expr => $v:expr),* $(,)?) => {{
    std::collections::HashMap::from([$(($k, $v),)*])
  }};
}

macro_rules! mdattrs {
  ($($key:ident = $value:expr),+) => {
    $crate::model::Attributes::from([
      $((stringify!($key).into(), $crate::model::value::Value::from($value)),)+
    ])
  };
}

macro_rules! mdnode {
  ($kind:expr, $range:expr $(,$($key:ident = $value:expr),*)?) => {
    $crate::model::node::NodeRef::from($crate::model::node::Node {
      kind: $kind,
      location: $range.into(),
      $(attributes: Some($crate::model::Attributes::from([
        $((stringify!($key).into(), $crate::model::value::Value::from($value)),)+
      ])),)?
      ..$crate::model::node::Node::default()
    })
  };

  ($kind:expr, $range:expr, $attrs:expr) => {
    $crate::model::node::NodeRef::from($crate::model::node::Node {
      kind: $kind,
      location: $range.into(),
      attributes: $attrs.into(),
      ..$crate::model::node::Node::default()
    })
  };
}