
macro_rules! mdattrs {
  ($($key:ident = $value:expr),+) => {
    $crate::model::Attributes::from([
      $((stringify!($key).into(), $crate::model::Value::from($value)),)+
    ])
  };
}

macro_rules! mdnode {
  ($kind:expr, $range:expr $(,$($key:ident = $value:expr),*)?) => {
    $crate::model::NodeRef::from($crate::model::Node {
      kind: $kind,
      location: $range.into(),
      $(attributes: Some($crate::model::Attributes::from([
        $((stringify!($key).into(), $crate::model::Value::from($value)),)+
      ])),)?
      ..$crate::model::Node::default()
    })
  };

  ($kind:expr, $range:expr, $attrs:expr) => {
    $crate::model::NodeRef::from($crate::model::Node {
      kind: $kind,
      location: $range.into(),
      attributes: $attrs.into(),
      ..$crate::model::Node::default()
    })
  };
}