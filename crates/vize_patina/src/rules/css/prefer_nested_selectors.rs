//! css/prefer-nested-selectors
//!
//! Recommend using CSS nesting for descendant selectors.

use std::borrow::Cow;

use lightningcss::stylesheet::StyleSheet;
use vize_s0::String;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{CssLintResult, CssRule, CssRuleMeta};

static META: CssRuleMeta = CssRuleMeta {
    name: "css/prefer-nested-selectors",
    description: "Recommend using CSS nesting for descendant selectors",
    default_severity: Severity::Warning,
};

/// Prefer nested selectors rule
pub struct PreferNestedSelectors;

impl CssRule for PreferNestedSelectors {
    fn meta(&self) -> &'static CssRuleMeta {
        &META
    }

    fn check<'i>(
        &self,
        source: &'i str,
        _stylesheet: &StyleSheet<'i>,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        scan(source, offset, result);
    }
}

/// At-rules whose body holds no style rules, so the whole block is skipped.
const NON_NESTED_BLOCK_AT_RULES: &[&str] = &[
    "keyframes",
    "-webkit-keyframes",
    "-moz-keyframes",
    "font-face",
    "page",
    "counter-style",
    "property",
    "font-feature-values",
    "color-profile",
    "viewport",
];

/// What an open brace opened.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Frame {
    /// A style rule. Everything inside it is *already nested*, so a
    /// descendant selector there is not worth reporting — there is
    /// nothing further to nest it into.
    Style,
    /// A conditional group at-rule (`@media`, `@supports`, `@container`,
    /// `@layer`). Its body sits at the nesting level of the at-rule
    /// itself, so a descendant selector inside one still reports.
    Group,
}

/// Walk the stylesheet's braces, reporting a descendant selector only
/// where nesting it would be an improvement.
///
/// The scan keeps a prelude running from the last boundary (`{`, `}` or
/// `;`) to the next `{`. Treating a declaration as a boundary is what
/// separates a selector from the text before it: without that,
/// `.a { color: red; .b {} }` reads `color: red;\n\n  .b` as one
/// selector, finds a space in it, and reports the nesting it is meant to
/// be recommending. Tracking [`Frame::Style`] is the other half — inside
/// a style rule the author has already nested.
fn scan(source: &str, offset: usize, result: &mut CssLintResult) {
    let bytes = source.as_bytes();
    let mut frames: Vec<Frame> = Vec::new();
    let mut comments = Vec::new();
    let mut prelude_start = 0usize;
    let mut i = 0usize;
    while let Some(&byte) = bytes.get(i) {
        match byte {
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                let end = skip_comment(bytes, i);
                comments.push((i, end));
                i = end;
                continue;
            }
            quote @ (b'"' | b'\'') => {
                i = skip_string(bytes, i, quote);
                continue;
            }
            b';' => {
                i += 1;
                prelude_start = i;
                comments.clear();
            }
            b'}' => {
                frames.pop();
                i += 1;
                prelude_start = i;
                comments.clear();
            }
            b'{' => {
                let (prelude, start, end) = selector_prelude(source, prelude_start, i, &comments);
                if let Some(keyword) = at_keyword(&prelude) {
                    if is_opaque_at_rule(keyword) {
                        i = skip_balanced_block(bytes, i);
                        prelude_start = i;
                        comments.clear();
                        continue;
                    }
                    frames.push(Frame::Group);
                } else {
                    if !frames.contains(&Frame::Style)
                        && !prelude.is_empty()
                        && !is_already_nested(&prelude)
                        && split_descendant_selector(&prelude).is_some()
                    {
                        report(start, end, offset, result);
                    }
                    frames.push(Frame::Style);
                }
                i += 1;
                prelude_start = i;
                comments.clear();
            }
            _ => i += 1,
        }
    }
}

/// CSS comments are removed before tokenization. Keep their original byte
/// ranges for diagnostics, but never treat their text or surrounding gap as
/// part of a selector combinator.
fn selector_prelude<'a>(
    source: &'a str,
    start: usize,
    end: usize,
    comments: &[(usize, usize)],
) -> (Cow<'a, str>, usize, usize) {
    if comments.is_empty() {
        let raw = source.get(start..end).unwrap_or_default();
        return (
            Cow::Borrowed(raw.trim()),
            start + raw.len() - raw.trim_start().len(),
            start + raw.trim_end().len(),
        );
    }

    let mut selector = String::with_capacity(end - start);
    let mut first = None;
    let mut last = start;
    let mut cursor = start;
    for &(comment_start, comment_end) in comments {
        append_prelude_segment(
            source,
            cursor,
            comment_start,
            &mut selector,
            &mut first,
            &mut last,
        );
        cursor = comment_end;
    }
    append_prelude_segment(source, cursor, end, &mut selector, &mut first, &mut last);
    (
        Cow::Owned(selector.trim().to_owned()),
        first.unwrap_or(end),
        last,
    )
}

fn append_prelude_segment(
    source: &str,
    start: usize,
    end: usize,
    selector: &mut String,
    first: &mut Option<usize>,
    last: &mut usize,
) {
    let segment = source.get(start..end).unwrap_or_default();
    selector.push_str(segment);
    if !segment.trim().is_empty() {
        if first.is_none() {
            *first = Some(start + segment.len() - segment.trim_start().len());
        }
        *last = start + segment.trim_end().len();
    }
}

fn report(start: usize, end: usize, offset: usize, result: &mut CssLintResult) {
    result.add_diagnostic(
        LintDiagnostic::warn(
            META.name,
            "Consider using CSS nesting for descendant selectors",
            u32::try_from(offset + start).unwrap_or(u32::MAX),
            u32::try_from(offset + end).unwrap_or(u32::MAX),
        )
        .with_help("Use CSS nesting syntax to nest child selectors inside parent selectors"),
    );
}

fn skip_comment(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 2;
    while i + 1 < bytes.len() {
        if bytes.get(i..i + 2) == Some(b"*/".as_slice()) {
            return i + 2;
        }
        i += 1;
    }
    bytes.len()
}

fn skip_string(bytes: &[u8], start: usize, quote: u8) -> usize {
    let mut i = start + 1;
    while let Some(&byte) = bytes.get(i) {
        match byte {
            b'\\' => i += 2,
            byte if byte == quote => return i + 1,
            _ => i += 1,
        }
    }
    bytes.len()
}

/// The at-rule keyword a prelude opens with, or `None` when it is a
/// selector.
fn at_keyword(prelude: &str) -> Option<&str> {
    let rest = prelude.strip_prefix('@')?;
    let end = rest
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
        .unwrap_or(rest.len());
    rest.get(..end).filter(|keyword| !keyword.is_empty())
}

fn is_opaque_at_rule(keyword: &str) -> bool {
    NON_NESTED_BLOCK_AT_RULES
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(keyword))
}

fn skip_balanced_block(bytes: &[u8], open_pos: usize) -> usize {
    let mut depth: i32 = 0;
    let mut i = open_pos;
    while let Some(&byte) = bytes.get(i) {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    bytes.len()
}

fn is_already_nested(selector: &str) -> bool {
    let bytes = selector.as_bytes();
    let (mut bracket, mut paren) = (0usize, 0usize);
    let (mut in_q, mut qc) = (false, 0u8);
    for &b in bytes {
        if !in_q && (b == b'"' || b == b'\'') {
            in_q = true;
            qc = b;
            continue;
        }
        if in_q {
            if b == qc {
                in_q = false;
            }
            continue;
        }
        match b {
            b'[' => bracket += 1,
            b']' => bracket = bracket.saturating_sub(1),
            b'(' => paren += 1,
            b')' => paren = paren.saturating_sub(1),
            b'&' if bracket == 0 && paren == 0 => return true,
            _ => {}
        }
    }
    false
}

fn split_descendant_selector(selector: &str) -> Option<(&str, &str)> {
    let bytes = selector.as_bytes();
    let (mut bracket, mut paren) = (0usize, 0usize);
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'[' => bracket += 1,
            b']' => bracket = bracket.saturating_sub(1),
            b'(' => paren += 1,
            b')' => paren = paren.saturating_sub(1),
            b' ' | b'>' | b'+' | b'~' if bracket == 0 && paren == 0 => {
                let (parent, child) = selector.split_at_checked(i)?;
                let parent = parent.trim();
                let child = child.trim();
                // A separator between entries in a selector list is not
                // a descendant combinator (e.g. `.a, .b` or `.a , .b`).
                if parent.ends_with(',') || child.starts_with(',') {
                    continue;
                }
                let child = child.trim_start_matches([' ', '>', '+', '~']).trim();
                if !parent.is_empty() && !child.is_empty() {
                    return Some((parent, child));
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests;
