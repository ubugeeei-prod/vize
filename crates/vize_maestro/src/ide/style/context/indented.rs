//! Cursor classification for indentation syntaxes (`lang="sass"`, `"stylus"`).
//!
//! Rules have no braces: an indented line under a selector is a declaration,
//! written `name: value` (Sass, Stylus) or `name value` (Stylus). Only the
//! cursor's own line is read.

use super::{
    CssContext, MAX_LOOKBACK,
    lex::{MAX_LOOKAHEAD, token_span, trivia_end},
};
use crate::ide::style::data;

pub(in crate::ide::style) fn at(content: &str, offset: usize) -> CssContext<'_> {
    classify(content, offset).unwrap_or(CssContext::Unknown)
}

fn classify(content: &str, offset: usize) -> Option<CssContext<'_>> {
    if offset > MAX_LOOKBACK || !content.is_char_boundary(offset) {
        return None;
    }
    let bytes = content.as_bytes();
    let line_start = content
        .get(..offset)?
        .rfind('\n')
        .map_or(0, |index| index + 1);
    let head = content.get(line_start..offset)?;
    let body_start = line_start + (head.len() - head.trim_start().len());
    if head.contains("//") || head.contains("/*") {
        return None;
    }
    let (token_start, token_end) = token_span(bytes, offset)?;
    let previous = token_start
        .checked_sub(1)
        .and_then(|index| bytes.get(index));
    if previous == Some(&b'@') && token_start - 1 == body_start {
        return Some(CssContext::AtRule {
            prefix: content.get(token_start - 1..offset)?,
            span: (token_start - 1, token_end),
        });
    }
    let line_head = content.get(body_start..token_start)?;
    if body_start > line_start {
        if let Some(property) = declaration_name(line_head) {
            return Some(CssContext::Value {
                property,
                prefix: content.get(token_start..offset)?,
                span: (token_start, token_end),
            });
        }
        if line_head.is_empty() {
            let limit = token_end.saturating_add(MAX_LOOKAHEAD).min(bytes.len());
            let next = trivia_end(bytes, token_end, limit).unwrap_or(limit);
            return Some(CssContext::Property {
                prefix: content.get(token_start..offset)?,
                span: (token_start, token_end),
                has_colon: bytes.get(next) == Some(&b':'),
            });
        }
    }
    if previous == Some(&b':') {
        let mut start = token_start - 1;
        if start > body_start && bytes.get(start - 1) == Some(&b':') {
            start -= 1;
        }
        return Some(CssContext::Pseudo {
            prefix: content.get(start..offset)?,
            span: (start, token_end),
        });
    }
    None
}

/// `color: ` or Stylus `color ` before the value token, for a known property.
fn declaration_name(line_head: &str) -> Option<&str> {
    let end = line_head.find(|ch: char| ch == ':' || ch.is_ascii_whitespace())?;
    let name = line_head.get(..end)?;
    let rest = line_head.get(end..)?.trim_start_matches(':');
    let separated = rest.is_empty() || rest.starts_with(|ch: char| ch.is_ascii_whitespace());
    (separated && data::property(name).is_some()).then_some(name)
}

#[cfg(test)]
mod tests {
    use super::{CssContext, at};

    fn classify(marked: &str) -> CssContext<'_> {
        let offset = marked.find('|').unwrap();
        // The cursor marker sits at the end of each probe, so slicing it off keeps offsets.
        at(marked.get(..offset).unwrap(), offset)
    }

    #[test]
    fn indented_declarations_complete_properties_and_values() {
        assert_eq!(
            classify(".card\n  col|"),
            CssContext::Property {
                prefix: "col",
                span: (8, 11),
                has_colon: false
            }
        );
        // Sass `name: value` and Stylus `name value`.
        for marked in [".card\n  color: re|", ".card\n  color re|"] {
            let CssContext::Value {
                property, prefix, ..
            } = classify(marked)
            else {
                panic!("expected a value context for {marked:?}");
            };
            assert_eq!((property, prefix), ("color", "re"));
        }
    }

    #[test]
    fn indented_selectors_complete_pseudos_and_at_rules() {
        assert_eq!(
            classify(".card\n  &:hov|"),
            CssContext::Pseudo {
                prefix: ":hov",
                span: (9, 13)
            }
        );
        assert_eq!(
            classify("@med|"),
            CssContext::AtRule {
                prefix: "@med",
                span: (0, 4)
            }
        );
        // A top-level selector line is not a declaration.
        assert_eq!(classify("car|"), CssContext::Unknown);
        assert_eq!(classify(".card\n  // col|"), CssContext::Unknown);
    }
}
