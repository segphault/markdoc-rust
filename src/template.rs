use crate::convert::convert_tag;
use crate::markdown::scan_markdoc_tag_end;
use crate::model::{Token, Type};
use crate::tag::Tag;

fn text_token(content: &str) -> Token {
  Token::Append {
    kind: Type::Text,
    attributes: Some([("content".into(), content.into())].into()),
  }
}

pub fn parse(content: &str) -> Vec<Token> {
  let mut output = vec![];
  let mut last = 0;

  for (pos, _) in content.match_indices("{%") {
    if let Some(length) = scan_markdoc_tag_end(content[pos..].as_bytes()) {
      let end = pos + length;
      let tag_content = &content[pos..end];
      let tag = Tag::from(tag_content);

      let line_start = content[..pos].rfind('\n').unwrap_or(0);
      let line_end = content[end..]
        .find('\n')
        .map(|x| x + end)
        .unwrap_or(content.len());

      let inline = content[line_start..line_end].trim() != tag_content;
      let preceding_end = if inline { pos } else { line_start };

      output.push(text_token(&content[last..preceding_end]));
      output.push(convert_tag(tag, true)); //inline));

      last = end;
    }
  }

  output.push(text_token(&content[last..]));
  output
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn basic_template_parsing() {
    let output = parse("this is a {% foo blah=\"asdf\" %}test{% /foo %} of template parsing");

    assert_eq!(
      output,
      vec![
        text_token("this is a "),
        Token::Open {
          kind: Type::Tag {
            name: "foo".into(),
            inline: true
          },
          attributes: Some([("blah".into(), "asdf".into())].into())
        },
        text_token("test"),
        Token::Close {
          kind: Type::Tag {
            name: "foo".into(),
            inline: true
          }
        },
        text_token(" of template parsing")
      ]
    )
  }

  #[test]
  fn template_parsing_with_block_tag() {
    let output = parse("this is a test\n{% foo blah=\"asdf\" %}\ntest\n{% /foo %}\n");

    assert_eq!(
      output,
      vec![
        text_token("this is a test"),
        Token::Open {
          kind: Type::Tag {
            name: "foo".into(),
            inline: true // todo(compat): false
          },
          attributes: Some([("blah".into(), "asdf".into())].into())
        },
        text_token("\ntest"),
        Token::Close {
          kind: Type::Tag {
            name: "foo".into(),
            inline: true // todo(compat): false
          }
        },
        text_token("\n")
      ]
    )
  }
}
