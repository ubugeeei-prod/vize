use std::borrow::Cow;

/// Strip JS/TS comments while preserving string literals.
///
/// The common path for template expressions has no comments, so the function
/// returns `Cow::Borrowed` without allocating. The owned buffer is reserved only
/// after the first line/block comment is actually found; until then the scanner
/// just walks bytes and keeps string/template literals intact.
pub fn strip_js_comments(expr: &str) -> Cow<'_, str> {
    strip_comments(expr, false).0
}

/// The same legacy comment transform, with an opt-in authored-byte witness.
/// `None` is the identity map; synthetic replacement spaces use `u32::MAX`.
pub(super) fn strip_comments_with_offsets(expr: &str) -> (Cow<'_, str>, Option<Vec<u32>>) {
    strip_comments(expr, true)
}

fn strip_comments(expr: &str, offsets: bool) -> (Cow<'_, str>, Option<Vec<u32>>) {
    let bytes = expr.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut changed = false;
    let mut mapping = None;
    #[expect(clippy::disallowed_types, reason = "Cow<str> owns a std String")]
    let mut out = std::string::String::new();

    while let Some(&c) = bytes.get(i) {
        if c == b'\'' || c == b'"' || c == b'`' {
            let literal_start = i;
            let quote = c;
            i += 1;

            while let Some(&current) = bytes.get(i) {
                i += 1;

                if current == b'\\' {
                    i = (i + 1).min(len);
                    continue;
                }

                if current == quote {
                    break;
                }
            }

            if changed {
                out.push_str(expr.get(literal_start..i).unwrap_or_default());
                if let Some(mapping) = mapping.as_mut() {
                    mapping.extend((literal_start..i).map(|at| at as u32));
                }
            }
            continue;
        }

        if c == b'/'
            && let Some(&next) = bytes.get(i + 1)
            && !is_escaped(bytes, i)
        {
            if next == b'/' {
                if !changed {
                    out.reserve(expr.len());
                    out.push_str(expr.get(..i).unwrap_or_default());
                    if offsets {
                        mapping = Some((0..i).map(|at| at as u32).collect::<Vec<_>>());
                    }
                    changed = true;
                }

                i += 2;
                while i < len && bytes.get(i) != Some(&b'\n') {
                    i += 1;
                }
                if i < len && bytes.get(i) == Some(&b'\n') {
                    out.push('\n');
                    if let Some(mapping) = mapping.as_mut() {
                        mapping.push(i as u32);
                    }
                    i += 1;
                }
                continue;
            }

            if next == b'*' {
                if !changed {
                    out.reserve(expr.len());
                    out.push_str(expr.get(..i).unwrap_or_default());
                    if offsets {
                        mapping = Some((0..i).map(|at| at as u32).collect::<Vec<_>>());
                    }
                    changed = true;
                }

                i += 2;
                while i + 1 < len
                    && !(bytes.get(i) == Some(&b'*') && bytes.get(i + 1) == Some(&b'/'))
                {
                    if bytes.get(i) == Some(&b'\n') {
                        out.push('\n');
                        if let Some(mapping) = mapping.as_mut() {
                            mapping.push(i as u32);
                        }
                    }
                    i += 1;
                }
                if i + 1 < len {
                    i += 2;
                } else {
                    i = len;
                }
                out.push(' ');
                if let Some(mapping) = mapping.as_mut() {
                    mapping.push(u32::MAX);
                }
                continue;
            }
        }

        if changed {
            let Some(ch) = expr.get(i..).and_then(|rest| rest.chars().next()) else {
                break;
            };
            out.push(ch);
            if let Some(mapping) = mapping.as_mut() {
                mapping.extend((i..i + ch.len_utf8()).map(|at| at as u32));
            }
            i += ch.len_utf8();
        } else {
            i += 1;
        }
    }

    if changed {
        (Cow::Owned(out), mapping)
    } else {
        (Cow::Borrowed(expr), None)
    }
}

fn is_escaped(bytes: &[u8], index: usize) -> bool {
    let mut cursor = index;
    while cursor > 0 && bytes.get(cursor - 1) == Some(&b'\\') {
        cursor -= 1;
    }
    (index - cursor) % 2 == 1
}
