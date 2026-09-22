//! Authored spans for the S4 string-plan emitter (Davinci P3-9).
//!
//! S2 ops keep the span of every element, attribute, binding, text, and
//! expression. The tokens inside them are located by the authored template
//! syntax, and each locator verifies the token it found, so the plan lane
//! anchors exactly the tokens the AST walker anchors.

use vize_s0::Span;
use vize_s2::expr::ExprRef;

/// `span` without surrounding whitespace, as the shipped parser trims an
/// interpolation's content.
pub(super) fn trimmed(source: &str, span: Span) -> Span {
    let Some(raw) = source.get(span.start as usize..span.end as usize) else {
        return span;
    };
    let lead = (raw.len() - raw.trim_start().len()) as u32;
    let tail = (raw.len() - raw.trim_end().len()) as u32;
    Span::new(span.start + lead, (span.end - tail).max(span.start + lead))
}

/// The content of a `{{ … }}` span, trimmed.
pub(super) fn interpolation_content(source: &str, span: Span) -> Option<Span> {
    let raw = source.get(span.start as usize..span.end as usize)?;
    (raw.starts_with("{{") && raw.ends_with("}}") && raw.len() >= 4)
        .then(|| trimmed(source, Span::new(span.start + 2, span.end - 2)))
}

/// The authored span of a retained or opaque expression.
pub(super) fn expression_span(expr: &ExprRef<'_>) -> Option<Span> {
    match expr {
        ExprRef::Js(js) => Some(js.span),
        ExprRef::Opaque(opaque) => Some(opaque.span),
        _ => None,
    }
}

/// Start of a literal attribute value in `name = "value"` authored at
/// `span`: after the `=`, whitespace, and an optional quote.
pub(super) fn attribute_value_start(source: &str, span: Span, name: &str) -> Option<u32> {
    let raw = source.get(span.start as usize..span.end as usize)?;
    let rest = raw
        .strip_prefix(name)?
        .trim_start()
        .strip_prefix('=')?
        .trim_start();
    let rest = rest.strip_prefix(['"', '\'']).unwrap_or(rest);
    Some(span.start + (raw.len() - rest.len()) as u32)
}

/// Start of a static `v-bind` argument in the directive authored at `span`
/// (`:name`, `v-bind:name`).
pub(super) fn argument_start(source: &str, span: Span, name: &str) -> Option<u32> {
    let raw = source.get(span.start as usize..span.end as usize)?;
    let rest = ["v-bind:", ":"]
        .iter()
        .find_map(|prefix| raw.strip_prefix(prefix))?;
    rest.starts_with(name)
        .then(|| span.start + (raw.len() - rest.len()) as u32)
}

/// The authored value of the directive at `span` (inside its quotes,
/// untrimmed), as the AST walker spans a directive expression.
pub(super) fn directive_value(source: &str, span: Span) -> Option<Span> {
    let raw = source.get(span.start as usize..span.end as usize)?;
    let equals = raw.find('=')?;
    let after = &raw[equals + 1..];
    let value = after.trim_start();
    let start = raw.len() - value.len();
    let (open, inner) = match value.as_bytes().first() {
        Some(&quote @ (b'"' | b'\'')) => (1, value[1..].strip_suffix(quote as char)?),
        _ => (0, value),
    };
    let start = span.start + (start + open) as u32;
    Some(Span::new(start, start + inner.len() as u32))
}

/// `span` widened over surrounding whitespace to the directive's quotes, as
/// the AST walker spans a directive value (` x ` in `v-if=" x "`). A span
/// that is not a whole quoted value is returned as is.
pub(super) fn quoted_value(source: &str, span: Span) -> Span {
    let bytes = source.as_bytes();
    let (mut start, mut end) = (span.start as usize, span.end as usize);
    while start > 0 && bytes.get(start - 1).is_some_and(u8::is_ascii_whitespace) {
        start -= 1;
    }
    while bytes.get(end).is_some_and(u8::is_ascii_whitespace) {
        end += 1;
    }
    let quote = start.checked_sub(1).and_then(|open| bytes.get(open));
    match (quote, bytes.get(end)) {
        (Some(open @ (b'"' | b'\'')), Some(close)) if open == close => {
            Span::new(start as u32, end as u32)
        }
        _ => span,
    }
}
