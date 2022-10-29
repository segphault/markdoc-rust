use super::Attributes;
use pulldown_cmark::CowStr;

#[derive(Debug)]
pub enum Renderable<'a> {
  Tag {
    name: CowStr<'a>,
    attributes: Option<Attributes<'a>>,
    children: Option<Vec<Renderable<'a>>>,
  },
  Fragment(Vec<Renderable<'a>>),
  String(CowStr<'a>),
  Null,
}
