//! JavaScript comments inside template expressions.
//!
//! vue-eslint-parser lists comments in directive values and interpolations
//! with the template's HTML comments, and eslint-plugin-vue's
//! `comment-directive` reads `eslint-disable` pragmas from all of them, so
//! `// eslint-disable-next-line` inside a `:class` array suppresses the next
//! line.

/// Each comment body in `expression` with its offset in `expression`.
/// Strings and template literal text are skipped.
pub(super) fn comments(expression: &str) -> Vec<(&str, usize)> {
    let bytes = expression.as_bytes();
    let mut found = Vec::new();
    // Open template literals; each entry counts the `{` nesting of its `${`.
    let mut templates: Vec<usize> = Vec::new();
    let mut in_template_text = false;
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        if in_template_text {
            match byte {
                b'\\' => index += 1,
                b'`' => {
                    in_template_text = false;
                    templates.pop();
                }
                b'$' if bytes.get(index + 1) == Some(&b'{') => {
                    in_template_text = false;
                    index += 1;
                }
                _ => {}
            }
            index += 1;
            continue;
        }
        match byte {
            b'\'' | b'"' => index = string_end(bytes, index),
            b'`' => {
                templates.push(0);
                in_template_text = true;
            }
            b'{' => {
                if let Some(depth) = templates.last_mut() {
                    *depth += 1;
                }
            }
            b'}' => match templates.last_mut() {
                Some(0) => in_template_text = true,
                Some(depth) => *depth -= 1,
                None => {}
            },
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                let end = memchr::memchr(b'\n', bytes.get(index..).unwrap_or_default())
                    .map_or(bytes.len(), |length| index + length);
                if let Some(body) = expression.get(index + 2..end) {
                    found.push((body, index));
                }
                index = end;
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                let end = expression
                    .get(index + 2..)
                    .and_then(|rest| rest.find("*/"))
                    .map_or(bytes.len(), |length| index + 2 + length);
                if let Some(body) = expression.get(index + 2..end) {
                    found.push((body, index));
                }
                index = end + 1;
            }
            _ => {}
        }
        index += 1;
    }
    found
}

/// The index of the quote closing the string opened at `open`.
fn string_end(bytes: &[u8], open: usize) -> usize {
    let quote = bytes.get(open).copied();
    let mut index = open + 1;
    while let Some(&byte) = bytes.get(index) {
        if byte == b'\\' {
            index += 1;
        } else if Some(byte) == quote || byte == b'\n' {
            return index;
        }
        index += 1;
    }
    bytes.len()
}

#[cfg(test)]
mod tests {
    use super::comments;

    #[test]
    fn finds_line_and_block_comments_outside_strings() {
        let expression = "[\n  ...a, // eslint-disable-next-line x\n  'not // a comment', `t ${b /* in */} //t`,\n  /* eslint-disable y */ c,\n]";
        assert_eq!(
            comments(expression)
                .into_iter()
                .map(|(body, at)| (body, expression.get(at..at + 2).unwrap_or_default()))
                .collect::<Vec<_>>(),
            [
                (" eslint-disable-next-line x", "//"),
                (" in ", "/*"),
                (" eslint-disable y ", "/*"),
            ]
        );
    }
}
