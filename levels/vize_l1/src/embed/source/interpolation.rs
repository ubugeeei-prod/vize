//! Vue interpolation prepares authored edges before the sole language parse.

use vize_l0::{Allocator, Span};

use super::{EmbedSource, SourceError, checked_add, checked_span, prepare_decoded_value};
use crate::markup::entity::EntityContext;

/// Select Vue's authored HTML-whitespace window, then decode it once as text.
///
/// `content` excludes the existing interpolation delimiters. Only TAB, LF, FF,
/// CR and SPACE are removed from authored edges; entity-produced whitespace,
/// NBSP and BOM remain. The result is the complete parse and emission source,
/// not an AST/comment minimum envelope. The caller retains the full construct
/// span separately, and must not remove source bytes after parsing this window.
/// Source preparation does not establish backend spelling/grammar admission.
pub fn prepare_vue_interpolation_in<'a>(
    allocator: &'a Allocator,
    file: &'a str,
    content: Span,
) -> Result<EmbedSource<'a>, SourceError> {
    let original = EmbedSource::authored(file, content)?;
    let raw = original.text();
    let start = raw
        .bytes()
        .take_while(|byte| html_whitespace(*byte))
        .count();
    let tail = raw.get(start..).ok_or(SourceError::InvalidAuthoredSpan)?;
    let end = raw
        .len()
        .checked_sub(
            tail.bytes()
                .rev()
                .take_while(|byte| html_whitespace(*byte))
                .count(),
        )
        .ok_or(SourceError::InvalidAuthoredSpan)?;
    let span = checked_span(
        checked_add(content.start as usize, start)?,
        checked_add(content.start as usize, end)?,
    )?;
    prepare_decoded_value(allocator, file, span, EntityContext::Text)
}

const fn html_whitespace(byte: u8) -> bool {
    matches!(byte, b'\t' | b'\n' | b'\x0c' | b'\r' | b' ')
}
