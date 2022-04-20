use pulldown_cmark::{Event, OffsetIter, Options, Parser};
use std::ops::Range;

pub type Events<'a> = Vec<(Event<'a>, Range<usize>)>;

pub fn tokenize(input: &str) -> OffsetIter<'_, '_> {
  let mut opts = Options::empty();
  opts.insert(Options::ENABLE_TABLES);
  opts.insert(Options::ENABLE_MARKDOC_TAGS);
  opts.insert(Options::DISABLE_INDENTED_CODE_BLOCKS);
  Parser::new_ext(input, opts).into_offset_iter()
}
