use pulldown_cmark::CowStr;
use std::collections::HashMap;

pub mod node;
pub mod schema;
pub mod value;
pub mod render;

pub type Attributes<'a> = HashMap<CowStr<'a>, value::Value<'a>>;