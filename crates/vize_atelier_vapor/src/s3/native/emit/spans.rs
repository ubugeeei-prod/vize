//! Authored spans for the native S3 Vapor payload (Davinci P3-9).
//!
//! S3 operands keep the span of each element, attribute and binding; the
//! tokens inside them (tag name, attribute value, directive argument) are
//! located by the HTML/Vue syntax of that authored text, and each locator
//! verifies the token it found before anchoring it.

use vize_atelier_core::codegen::document::EmitDocument;
use vize_carton::Span;

use super::super::AuthoredSpan;
use super::Emitter;

impl Emitter<'_, '_> {
    /// Append `text` to a template, linked to the `authored_len` authored
    /// bytes at the start of `token` when a map is requested.
    pub(super) fn link(
        &self,
        template: &mut EmitDocument,
        text: &str,
        token: Option<AuthoredSpan>,
        authored_len: usize,
    ) {
        match token.filter(|_| self.source.is_some()) {
            Some((start, _)) => {
                let end = start + authored_len as u32;
                template.push_linked(text, Span::new(start, end));
            }
            None => template.push_str(text),
        }
    }

    /// The token `locate` finds inside the authored text of `span`.
    pub(super) fn token(
        &self,
        span: AuthoredSpan,
        locate: impl FnOnce(&str) -> Option<usize>,
    ) -> Option<AuthoredSpan> {
        let raw = self.source?.get(span.0 as usize..span.1 as usize)?;
        let offset = u32::try_from(locate(raw)?).ok()?;
        Some((span.0 + offset, span.1))
    }

    /// The authored expression inside `span`. Compound text records the whole
    /// `{{ expr }}`; a lone interpolation's span is already the expression.
    /// The legacy expression loc is that inner text (P3-9).
    pub(super) fn expression_anchor(&self, span: AuthoredSpan, text: &str) -> AuthoredSpan {
        let Some(raw) = self
            .source
            .and_then(|source| source.get(span.0 as usize..span.1 as usize))
        else {
            return span;
        };
        let inner = raw
            .strip_prefix("{{")
            .map(str::trim_start)
            .filter(|rest| rest.starts_with(text))
            .map(|rest| raw.len() - rest.len());
        let Some(offset) = inner.or_else(|| (raw.len() == text.len()).then_some(0)) else {
            return span;
        };
        let start = span.0 + offset as u32;
        (start, start + text.len() as u32)
    }

    /// `span` without the whitespace the payload value was trimmed of.
    pub(super) fn trimmed(&self, span: AuthoredSpan) -> AuthoredSpan {
        let Some(raw) = self
            .source
            .and_then(|s| s.get(span.0 as usize..span.1 as usize))
        else {
            return span;
        };
        let lead = (raw.len() - raw.trim_start().len()) as u32;
        let tail = (raw.len() - raw.trim_end().len()) as u32;
        (span.0 + lead, span.1 - tail)
    }
}

/// Offset of the tag name in an element's authored text: after its `<`.
pub(super) fn tag_offset(raw: &str, tag: &str) -> Option<usize> {
    raw.strip_prefix('<')?.starts_with(tag).then_some(1)
}

/// Offset of a literal attribute value in `name = "value"` authored text.
pub(super) fn value_offset(raw: &str, name: &str, value: &str) -> Option<usize> {
    let rest = raw
        .strip_prefix(name)?
        .trim_start()
        .strip_prefix('=')?
        .trim_start();
    let rest = rest.strip_prefix(['"', '\'']).unwrap_or(rest);
    rest.starts_with(value).then(|| raw.len() - rest.len())
}

/// Offset of a static argument in a `:name` / `v-bind:name` / `.name` /
/// `@name` / `v-on:name` directive's authored text.
pub(super) fn argument_offset(raw: &str, name: &str) -> Option<usize> {
    let rest = ["v-bind:", "v-on:", ":", ".", "@"]
        .iter()
        .find_map(|prefix| raw.strip_prefix(prefix))?;
    rest.starts_with(name).then(|| raw.len() - rest.len())
}
