//! Multiline interpolation rendering.
//!
//! Renders the lines of a formatted interpolation expression while keeping
//! multiline template-literal content verbatim, so a template literal's
//! semantically significant raw value is preserved and `vize fmt` stays
//! idempotent. (#3247)

use super::TemplateFormatter;
use crate::template::helpers::byte_at;
use crate::template::helpers::sub_slice;
use crate::{options::FormatOptions, script};
use vize_l0::{String, ToCompactString};

impl TemplateFormatter<'_> {
    /// Render the lines of a formatted interpolation expression at `depth + 1`,
    /// but emit any line that begins inside a multiline template-literal quasi
    /// verbatim. A template literal's raw string content is semantically
    /// significant (it is part of the rendered value), so re-indenting it would
    /// both corrupt the output and break idempotence: every `vize fmt` pass
    /// would prepend indentation again, so the content drifts further on each
    /// run. (#3247)
    pub(super) fn render_interpolation_expr_lines(&self, expr: &str, depth: usize) -> String {
        let trimmed = expr.trim();
        let quasi_line_starts = template_literal_quasi_line_starts(trimmed);
        let mut out = String::default();
        for (idx, line) in trimmed.lines().enumerate() {
            if quasi_line_starts.get(idx).copied().unwrap_or(false) {
                // Inside a template-literal quasi: preserve the bytes exactly.
                out.push_str(line);
            } else {
                self.write_indent_string(&mut out, depth + 1);
                out.push_str(line.trim_end_matches('\r'));
            }
            out.push_str(self.newline_str());
        }
        out
    }
}

/// For a formatted JS expression, return a per-line flag telling whether the
/// line's first byte lies inside a template-literal quasi (raw backtick-string
/// content). Index 0 is the first line and is always `false`, because a
/// trimmed expression begins in code. Callers use this to keep multiline
/// template-literal content verbatim when re-indenting interpolations, so the
/// literal's rendered value is preserved and formatting stays idempotent.
///
/// The scan tracks nested `${ … }` interpolations (which may themselves
/// contain template literals) and skips ordinary `'…'` / "…" strings and
/// backslash escapes so their braces and backticks do not confuse the state
/// machine. Regex literals are not modelled; a backtick inside one is rare in
/// template expressions and would at worst leave a line indented. (#3247)
fn template_literal_quasi_line_starts(expr: &str) -> Vec<bool> {
    enum Frame {
        /// Inside backticks, currently in quasi (raw string) text.
        Template,
        /// Inside `${ … }`; tracks `{`/`}` nesting depth within the interp.
        Interp(i32),
    }

    let bytes = expr.as_bytes();
    let mut starts = vec![false];
    let mut stack: Vec<Frame> = Vec::new();
    let mut in_str: Option<u8> = None;
    let mut i = 0;

    while let Some(&b) = bytes.get(i) {
        if b == b'\n' {
            starts.push(in_str.is_none() && matches!(stack.last(), Some(Frame::Template)));
            i += 1;
            continue;
        }

        if let Some(quote) = in_str {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == quote {
                in_str = None;
            }
            i += 1;
            continue;
        }

        match stack.last() {
            Some(Frame::Template) => match b {
                b'\\' => i += 2,
                b'`' => {
                    stack.pop();
                    i += 1;
                }
                b'$' if bytes.get(i + 1) == Some(&b'{') => {
                    stack.push(Frame::Interp(0));
                    i += 2;
                }
                _ => i += 1,
            },
            _ => match b {
                b'`' => {
                    stack.push(Frame::Template);
                    i += 1;
                }
                b'\'' | b'"' => {
                    in_str = Some(b);
                    i += 1;
                }
                b'{' => {
                    if let Some(Frame::Interp(d)) = stack.last_mut() {
                        *d += 1;
                    }
                    i += 1;
                }
                b'}' => {
                    if let Some(Frame::Interp(d)) = stack.last_mut() {
                        if *d == 0 {
                            stack.pop();
                        } else {
                            *d -= 1;
                        }
                    }
                    i += 1;
                }
                _ => i += 1,
            },
        }
    }

    starts
}

pub(super) fn parse_interpolation_range(
    source: &[u8],
    start: usize,
) -> Option<(usize, usize, usize)> {
    let len = source.len();
    if start + 1 >= len || byte_at(source, start) != b'{' || byte_at(source, start + 1) != b'{' {
        return None;
    }

    let expr_start = start + 2;
    let mut depth = 1;
    let mut pos = expr_start;

    while pos + 1 < len {
        if byte_at(source, pos) == b'{' && byte_at(source, pos + 1) == b'{' {
            depth += 1;
            pos += 2;
        } else if byte_at(source, pos) == b'}' && byte_at(source, pos + 1) == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some((expr_start, pos, pos + 2));
            }
            pos += 2;
        } else {
            pos += 1;
        }
    }

    None
}

/// Format interpolations in text content: `{{expr}}` -> `{{ expr }}`.
pub(crate) fn format_interpolations(text: &str, options: &FormatOptions) -> String {
    let bytes = text.as_bytes();
    let len = bytes.len();

    // Fast path: no `{` at all means no interpolations and no special bytes,
    // so the text is returned verbatim with a single allocation.
    let Some(first_brace) = memchr::memchr(b'{', bytes) else {
        return text.to_compact_string();
    };

    let mut result = String::with_capacity(len + 16);
    // Everything before the first `{` is ordinary text; copy it in one shot.
    result.push_str(text.get(..first_brace).unwrap_or_default());
    let mut pos = first_brace;

    while pos < len {
        if pos + 1 < len && byte_at(bytes, pos) == b'{' && byte_at(bytes, pos + 1) == b'{' {
            // Find closing }}
            let expr_start = pos + 2;
            let mut depth = 1;
            let mut expr_end = expr_start;

            while expr_end + 1 < len {
                if byte_at(bytes, expr_end) == b'{' && byte_at(bytes, expr_end + 1) == b'{' {
                    depth += 1;
                    expr_end += 2;
                } else if byte_at(bytes, expr_end) == b'}' && byte_at(bytes, expr_end + 1) == b'}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    expr_end += 2;
                } else {
                    expr_end += 1;
                }
            }

            if depth == 0 {
                let expr = text.get(expr_start..expr_end).unwrap_or_default();
                let formatted_expr = format_interpolation_expression(expr, options);
                result.push_str("{{ ");
                result.push_str(&formatted_expr);
                result.push_str(" }}");
                pos = expr_end + 2;
            } else {
                // Unclosed interpolation -- keep as-is
                result.push('{');
                pos += 1;
            }
        } else {
            // Ordinary text. Copy the run up to (but not including) the next
            // `{` in a single push instead of char-by-char. A lone `{` (one
            // not starting a `{{`) is emitted and stepped over individually,
            // exactly as before.
            let rest = sub_slice(bytes, pos + 1..);
            let next = memchr::memchr(b'{', rest).map_or(len, |off| pos + 1 + off);
            result.push_str(text.get(pos..next).unwrap_or_default());
            pos = next;
        }
    }

    result
}

pub(super) fn format_interpolation_expression(expr: &str, options: &FormatOptions) -> String {
    script::format_js_expression(expr, options).unwrap_or_else(|| expr.trim().to_compact_string())
}
