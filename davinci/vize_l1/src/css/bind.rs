//! CSS `v-bind()` calls in a style block, found the way Vue's compiler finds
//! them in every style language: strings and comments (`/* */` and `//`) are
//! not calls, `my-v-bind(` / `-webkit-v-bind(` need a left boundary, and an
//! unmatched `(` stops the scan.

use core::ops::Range;

/// One `v-bind(...)` call. Offsets are bytes into the scanned source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VBindCall<'a> {
    /// Offset of the `v` in `v-bind`.
    pub call_start: usize,
    /// Offset after the matching `)`.
    pub call_end: usize,
    /// Offset of the argument after whitespace and an outer quote.
    pub expression_start: usize,
    /// The argument, trimmed and without outer quotes: a slice of the source.
    pub expression: &'a str,
}

impl VBindCall<'_> {
    #[must_use]
    pub const fn expression_range(&self) -> Range<usize> {
        self.expression_start..self.expression_start + self.expression.len()
    }
}

/// Every call in source order, up to an unmatched `(`.
pub fn v_bind_calls(css: &str) -> impl Iterator<Item = VBindCall<'_>> {
    let mut from = 0;
    core::iter::from_fn(move || {
        let call = next(css, from)?;
        from = call.call_end;
        Some(call)
    })
}

fn next(css: &str, from: usize) -> Option<VBindCall<'_>> {
    let start = find_next_v_bind(
        css,
        from,
        #[cfg(feature = "style-bindings")]
        false,
    )?;
    let inner = start + 7;
    let after_open = css.get(inner..)?;
    let end = find_matching_paren(after_open)?;
    let inside = after_open.get(..end)?;
    let trimmed = inside.trim();
    let expression = trim_outer_quotes(trimmed);
    let quote = usize::from(expression.len() != trimmed.len());
    Some(VBindCall {
        call_start: start,
        call_end: inner + end + 1,
        expression_start: inner + inside.len() - inside.trim_start().len() + quote,
        expression,
    })
}

fn trim_outer_quotes(expr: &str) -> &str {
    ['"', '\'']
        .into_iter()
        .find_map(|quote| expr.strip_prefix(quote)?.strip_suffix(quote))
        .unwrap_or(expr)
}

fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth = 1u32;
    let mut quote = None;
    let mut escaped = false;
    let mut in_block_comment = false;
    let mut in_line_comment = false;
    let mut chars = s.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        if in_block_comment {
            if c == '*' && chars.peek().is_some_and(|(_, next)| *next == '/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }
        if in_line_comment {
            if c == '\n' || c == '\r' {
                in_line_comment = false;
            }
            continue;
        }
        if let Some(quote_char) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if c == '\\' {
                escaped = true;
                continue;
            }
            if c == quote_char {
                quote = None;
            }
            continue;
        }
        match c {
            '/' if chars.peek().is_some_and(|(_, next)| *next == '*') => {
                chars.next();
                in_block_comment = true;
            }
            '/' if chars.peek().is_some_and(|(_, next)| *next == '/') => {
                chars.next();
                in_line_comment = true;
            }
            '"' | '\'' | '`' => quote = Some(c),
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_next_v_bind(
    css: &str,
    start: usize,
    #[cfg(feature = "style-bindings")] allow_spaced: bool,
) -> Option<usize> {
    let bytes = css.as_bytes();
    let mut pos = start;
    let mut quote: Option<u8> = None;
    let mut escaped = false;
    let mut in_block_comment = false;
    let mut in_line_comment = false;

    while let Some(&byte) = bytes.get(pos) {
        if in_block_comment {
            if byte == b'*' && bytes.get(pos + 1) == Some(&b'/') {
                in_block_comment = false;
                pos += 2;
            } else {
                pos += 1;
            }
            continue;
        }
        if in_line_comment {
            if byte == b'\n' || byte == b'\r' {
                in_line_comment = false;
            }
            pos += 1;
            continue;
        }
        if let Some(quote_byte) = quote {
            if escaped {
                escaped = false;
                pos += 1;
                continue;
            }
            if byte == b'\\' {
                escaped = true;
                pos += 1;
                continue;
            }
            if byte == quote_byte {
                quote = None;
            }
            pos += 1;
            continue;
        }
        match byte {
            b'"' | b'\'' | b'`' => {
                quote = Some(byte);
                pos += 1;
            }
            b'/' if bytes.get(pos + 1) == Some(&b'*') => {
                in_block_comment = true;
                pos += 2;
            }
            b'/' if bytes.get(pos + 1) == Some(&b'/') => {
                in_line_comment = true;
                pos += 2;
            }
            b'v' if bytes
                .get(pos..)
                .is_some_and(|rest| rest.starts_with(b"v-bind("))
                && has_v_bind_left_boundary(bytes, pos) =>
            {
                return Some(pos);
            }
            #[cfg(feature = "style-bindings")]
            b'v' if allow_spaced
                && has_v_bind_left_boundary(bytes, pos)
                && css
                    .get(pos..)
                    .and_then(|rest| rest.strip_prefix("v-bind"))
                    .is_some_and(|tail| {
                        tail.trim_start_matches(binding_whitespace).starts_with('(')
                    }) =>
            {
                return Some(pos);
            }
            _ => pos += 1,
        }
    }
    None
}

fn has_v_bind_left_boundary(bytes: &[u8], pos: usize) -> bool {
    pos.checked_sub(1)
        .and_then(|before| bytes.get(before))
        .is_none_or(|&byte| !is_css_identifier_byte(byte))
}

fn is_css_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
}

#[cfg(feature = "style-bindings")]
fn binding_whitespace(c: char) -> bool {
    // ECMAScript \s includes the byte order mark and excludes next-line (U+0085).
    c == '\u{feff}' || (c != '\u{85}' && c.is_whitespace())
}

/// Byte offset of the first authored Vue CSS binding opener. It also covers
/// an unterminated call, where a checker cannot prove that the style is inert.
/// Comments, strings and CSS identifier suffixes follow the same lexical rules
/// as [`v_bind_calls`].
#[cfg(feature = "style-bindings")]
#[must_use]
pub fn first_v_bind_start(css: &str) -> Option<usize> {
    if css.contains("v-bind") {
        find_next_v_bind(css, 0, true)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "style-bindings")]
    #[test]
    fn checker_openers_allow_ecmascript_whitespace_and_preserve_lexical_boundaries() {
        use super::{first_v_bind_start, v_bind_calls};
        for gap in [
            "", " ", "\t", "\u{000b}", "\u{000c}", "\r\n", "\u{00a0}", "\u{1680}", "\u{2003}",
            "\u{2028}", "\u{2029}", "\u{3000}", "\u{feff}",
        ] {
            let prefix = "/* v-bind (ignored) */ \"v-bind (text)\" my-v-bind (suffix) ";
            let css = vize_l0::cstr!("{prefix}v-bind{gap}(incomplete");
            assert_eq!(first_v_bind_start(&css), Some(prefix.len()));
            // The original CSS lowering still reports complete contiguous calls.
            assert_eq!(v_bind_calls(&css).count(), 0);
        }
        for css in [
            "v-bind\u{0085}(not_js_space)",
            "v-bind /* comment */(x)",
            "'v-bind \\(text)'",
            "x-v-bind (x)",
            "v-binding (x)",
        ] {
            assert_eq!(first_v_bind_start(css), None, "{css}");
        }
    }

    use super::v_bind_calls;
    use alloc::vec::Vec;

    fn expressions(css: &str) -> Vec<(&str, &str)> {
        v_bind_calls(css)
            .map(|call| {
                let range = call.expression_range();
                (
                    css.get(call.call_start..call.call_end).unwrap_or_default(),
                    css.get(range).unwrap_or_default(),
                )
            })
            .collect()
    }

    #[test]
    fn calls_skip_strings_comments_and_unbounded_names() {
        assert_eq!(
            expressions(
                ".a { color: v-bind(color); }\n/* v-bind(block) */\n// v-bind(line)\n.b { content: 'v-bind(text)'; width: my-v-bind(x); height: v-bind( 'size.h' ); }\n.c { margin: v-bind(fn(a, (b))) }"
            ),
            [
                ("v-bind(color)", "color"),
                ("v-bind( 'size.h' )", "size.h"),
                ("v-bind(fn(a, (b)))", "fn(a, (b))"),
            ]
        );
    }

    #[test]
    fn an_unmatched_paren_ends_the_scan() {
        assert_eq!(
            expressions(".a { color: v-bind(a); width: v-bind(b; } .c { top: v-bind(c) }"),
            [("v-bind(a)", "a")]
        );
    }
}

#[cfg(feature = "original-style-bindings")]
mod authored;
#[cfg(feature = "original-style-bindings")]
pub use authored::{StyleBinding, binding_calls_in};
