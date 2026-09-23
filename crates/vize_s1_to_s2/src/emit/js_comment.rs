//! Safe raw-JS comment spelling for expression emit boundaries.

use vize_s0::expression_guard::scan::{
    keyword_allows_regex_after, skip_identifier, skip_line_comment, skip_number, skip_quoted,
    skip_regex,
};
use vize_s0::{Allocator, Span, String};
use vize_s2::expr::JsExpr;

#[derive(Clone)]
pub(super) enum RawJs<'a> {
    Borrowed(&'a str),
    Owned(String),
}

impl RawJs<'_> {
    pub(super) fn as_str(&self) -> &str {
        match self {
            Self::Borrowed(source) => source,
            Self::Owned(source) => source.as_str(),
        }
    }
}

pub(super) fn line_comment_source_as_block<'a>(
    source: &'a str,
    span_start: u32,
) -> Option<RawJs<'a>> {
    if !source.contains("//") {
        return None;
    }
    let converted = convert_line_comments_to_block(source);
    if converted == source || !source_is_js(converted.as_str(), span_start) {
        return None;
    }
    Some(RawJs::Owned(converted))
}

pub(super) fn source_is_js(source: &str, span_start: u32) -> bool {
    let allocator = Allocator::new();
    JsExpr::parse_in(
        &allocator,
        source,
        Span::new(span_start, span_start + source.len() as u32),
    )
    .is_ok()
}

pub(super) fn convert_line_comments_to_block(content: &str) -> String {
    let bytes = content.as_bytes();
    // Every scan stops on an ASCII delimiter or at the end, so these
    // ranges are whole chars of `content`.
    let text = |start: usize, end: usize| content.get(start..end).unwrap_or_default();
    let mut result = String::with_capacity(content.len());
    let mut can_start_regex = true;
    let mut i = 0;
    while let Some(&byte) = bytes.get(i) {
        match byte {
            b'\'' | b'"' | b'`' => {
                let end = skip_quoted(bytes, i + 1, byte).min(bytes.len());
                result.push_str(text(i, end));
                i = end;
                can_start_regex = false;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                let start = i + 2;
                let end = skip_line_comment(bytes, start);
                result.push_str("/* ");
                result.push_str(text(start, end).trim_end().replace("*/", "* /").as_str());
                result.push_str(" */");
                i = end;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                let end = bytes
                    .get(i + 2..)
                    .and_then(|rest| rest.windows(2).position(|pair| pair == b"*/"))
                    .map_or(bytes.len(), |at| i + 2 + at + 2);
                result.push_str(text(i, end));
                i = end;
            }
            b'/' if can_start_regex => {
                if let Some(end) = skip_regex(bytes, i + 1) {
                    result.push_str(text(i, end));
                    i = end;
                    can_start_regex = false;
                } else {
                    result.push('/');
                    i += 1;
                }
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' => {
                let end = skip_identifier(bytes, i + 1);
                result.push_str(text(i, end));
                can_start_regex = keyword_allows_regex_after(bytes.get(i..end).unwrap_or_default());
                i = end;
            }
            b'0'..=b'9' => {
                let end = skip_number(bytes, i + 1);
                result.push_str(text(i, end));
                i = end;
                can_start_regex = false;
            }
            b')' | b']' | b'}' => {
                result.push(char::from(byte));
                i += 1;
                can_start_regex = false;
            }
            b'+' | b'-' if bytes.get(i + 1) == Some(&byte) => {
                result.push_str(text(i, i + 2));
                i += 2;
                can_start_regex = false;
            }
            _ => {
                let ch = content
                    .get(i..)
                    .and_then(|rest| rest.chars().next())
                    .unwrap_or('\u{FFFD}');
                result.push(ch);
                i += ch.len_utf8();
                if !ch.is_ascii_whitespace() {
                    can_start_regex = true;
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::convert_line_comments_to_block;

    #[test]
    fn preserves_regex_strings_and_blocks_while_rewriting_line_comments() {
        assert_eq!(
            convert_line_comments_to_block("url.replace(/https?:\\/\\/[^/]+\\//, '//')"),
            "url.replace(/https?:\\/\\/[^/]+\\//, '//')"
        );
        assert_eq!(
            convert_line_comments_to_block("x /* // */ + y // */"),
            "x /* // */ + y /*  * / */"
        );
    }
}
