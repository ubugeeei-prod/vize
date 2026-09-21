//! Authored spans for the native S3 Vapor payload (Davinci P3-9).
//!
//! S3 operands keep the span of each element, attribute and binding; the
//! tokens inside them (tag name, attribute value, directive argument) are
//! located by the HTML/Vue syntax of that authored text, and each locator
//! verifies the token it found before anchoring it.

use vize_atelier_core::codegen::spanned::SpannedText;

use super::super::AuthoredSpan;
use super::Emitter;

impl Emitter<'_, '_> {
    /// Anchor the next template byte at an authored span's start.
    pub(super) fn mark(&self, template: &mut SpannedText, span: Option<AuthoredSpan>) {
        if let Some(span) = span.filter(|_| self.source.is_some()) {
            template.push_mapped("", span.0);
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
