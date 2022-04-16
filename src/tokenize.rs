use crate::markdown;
use crate::model::Token;

pub fn tokenize(input: &str) -> impl Iterator<Item = Token> + '_  {
  let mut opts = markdown::Options::empty();
  opts.insert(markdown::Options::ENABLE_TABLES);
  opts.insert(markdown::Options::ENABLE_MARKDOC_TAGS);
  opts.insert(markdown::Options::DISABLE_INDENTED_CODE_BLOCKS);
  markdown::Parser::new_ext(input, opts).map(Token::from)
}
