use crate::tokenize::Events;
use pulldown_cmark::{scan_markdoc_tag_end, Event};

pub fn parse(content: &str) -> Events {
  let mut output = vec![];
  let mut last = 0;

  for (pos, _) in content.match_indices("{%") {
    if let Some(length) = scan_markdoc_tag_end(content[pos..].as_bytes()) {
      let end = pos + length;
      let tag_content = &content[pos..end];

      let line_start = content[..pos].rfind('\n').unwrap_or(0);
      let line_end = content[end..]
        .find('\n')
        .map(|x| x + end)
        .unwrap_or(content.len());

      let inline = content[line_start..line_end].trim() != tag_content;
      let preceding_end = if inline { pos } else { line_start };

      output.push((
        Event::Text(content[last..preceding_end].into()),
        last..preceding_end,
      ));

      output.push((
        Event::MarkdocTag(tag_content.into(), /* todo(compat): inline */ true),
        pos..end,
      ));

      last = end;
    }
  }

  output.push((Event::Text(content[last..].into()), last..content.len()));
  output
}

// #[cfg(test)]
// mod tests {
//   use super::*;
//   use pretty_assertions::assert_eq;

//   #[test]
//   fn basic_template_parsing() {
//     let output = parse(
//       "this is a {% foo blah=\"asdf\" %}test{% /foo %} of template parsing",
//       0,
//     );

//     assert_eq!(
//       output,
//       vec![
//         text_token("this is a "),
//         Token::Open {
//           kind: Type::Tag {
//             name: "foo".into(),
//             inline: true
//           },
//           attributes: Some([("blah".into(), "asdf".into())].into())
//         },
//         text_token("test"),
//         Token::Close {
//           kind: Type::Tag {
//             name: "foo".into(),
//             inline: true
//           }
//         },
//         text_token(" of template parsing")
//       ]
//     )
//   }

//   #[test]
//   fn template_parsing_with_block_tag() {
//     let output = parse("this is a test\n{% foo blah=\"asdf\" %}\ntest\n{% /foo %}\n");

//     assert_eq!(
//       output,
//       vec![
//         text_token("this is a test"),
//         Token::Open {
//           kind: Type::Tag {
//             name: "foo".into(),
//             inline: true // todo(compat): false
//           },
//           attributes: Some([("blah".into(), "asdf".into())].into())
//         },
//         text_token("\ntest"),
//         Token::Close {
//           kind: Type::Tag {
//             name: "foo".into(),
//             inline: true // todo(compat): false
//           }
//         },
//         text_token("\n")
//       ]
//     )
//   }
// }
