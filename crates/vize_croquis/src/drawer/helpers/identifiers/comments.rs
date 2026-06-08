use std::borrow::Cow;

/// Strip JS/TS comments while preserving string literals.
///
/// The common path for template expressions has no comments, so the function
/// returns `Cow::Borrowed` without allocating. The owned buffer is reserved only
/// after the first line/block comment is actually found; until then the scanner
/// just walks bytes and keeps string/template literals intact.
pub fn strip_js_comments(expr: &str) -> Cow<'_, str> {
    let bytes = expr.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut changed = false;
    #[allow(clippy::disallowed_types)]
    let mut out = std::string::String::new();

    while i < len {
        let c = bytes[i];

        if c == b'\'' || c == b'"' || c == b'`' {
            let quote = c;
            if changed {
                out.push(quote as char);
            }
            i += 1;

            while i < len {
                let current = bytes[i];
                if changed {
                    out.push(current as char);
                }
                i += 1;

                if current == b'\\' {
                    if i < len {
                        if changed {
                            out.push(bytes[i] as char);
                        }
                        i += 1;
                    }
                    continue;
                }

                if current == quote {
                    break;
                }
            }

            continue;
        }

        if c == b'/' && i + 1 < len {
            let next = bytes[i + 1];

            if next == b'/' {
                if !changed {
                    out.reserve(expr.len());
                    out.push_str(&expr[..i]);
                    changed = true;
                }

                i += 2;
                while i < len && bytes[i] != b'\n' {
                    i += 1;
                }
                if i < len && bytes[i] == b'\n' {
                    out.push('\n');
                    i += 1;
                }
                continue;
            }

            if next == b'*' {
                if !changed {
                    out.reserve(expr.len());
                    out.push_str(&expr[..i]);
                    changed = true;
                }

                i += 2;
                while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    if bytes[i] == b'\n' {
                        out.push('\n');
                    }
                    i += 1;
                }
                if i + 1 < len {
                    i += 2;
                } else {
                    i = len;
                }
                out.push(' ');
                continue;
            }
        }

        if changed {
            out.push(c as char);
        }
        i += 1;
    }

    if changed {
        Cow::Owned(out)
    } else {
        Cow::Borrowed(expr)
    }
}
