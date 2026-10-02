//! Checked authored and decoded UTF-8 coordinates for L1 embedded syntax.
//!
//! HTML decoding happens once here, on construct-selected authored bytes. Language
//! providers only receive the resulting text. Exact projection never invents
//! positions inside an authored entity; diagnostic projection can conservatively
//! highlight the entire reference instead. Neither operation reparses syntax.

use vize_l0::{Allocator, Span, String, Vec};

use crate::markup::entity::{EntityContext, decode_one, needs_decoding};

mod map;
pub use map::{DecodeMap, DecodeSegment, DecodeSegmentKind};
mod interpolation;
pub use interpolation::prepare_vue_interpolation_in;

/// Invalid source coordinates or a request that splits one authored entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceError {
    InvalidAuthoredSpan,
    InvalidDecodedSpan,
    SourceTooLarge,
    InvalidMapCoverage,
    InvalidIdentitySegment,
    InvalidEntitySegment,
    PartialEntityBoundary,
}

/// Validated source retained with an embedded syntax artifact.
///
/// Plain text borrows its exact authored slice. Decoded text and its complete
/// map borrow the shared arena, while authored offsets remain file-absolute.
/// Private fields prevent consumers from pairing text with an unrelated map.
/// The retained input identifies a physical byte buffer, not a document,
/// grammar or producer. Shared and empty inputs need separate host identity.
#[derive(Debug, Clone, Copy)]
pub struct EmbedSource<'a> {
    authored_root: &'a str,
    span: Span,
    text: &'a str,
    decoded: Option<DecodeMap<'a>>,
}

impl<'a> EmbedSource<'a> {
    /// Raw source for a program, mustache or other non-HTML attribute grammar.
    /// This constructor deliberately does not interpret character references.
    pub fn authored(source: &'a str, span: Span) -> Result<Self, SourceError> {
        checked_len(source.len())?;
        let text = source
            .get(span.start as usize..span.end as usize)
            .ok_or(SourceError::InvalidAuthoredSpan)?;
        Ok(Self {
            authored_root: source,
            span,
            text,
            decoded: None,
        })
    }

    /// The complete input supplied to native source preparation, unchanged by
    /// decoding or slicing. Pointer and length can check buffer provenance;
    /// they cannot distinguish documents sharing that buffer, including empty
    /// inputs. Consumers must also retain the host's document key and version.
    #[must_use]
    pub const fn authored_root(self) -> &'a str {
        self.authored_root
    }

    #[must_use]
    pub const fn span(self) -> Span {
        self.span
    }

    #[must_use]
    pub const fn text(self) -> &'a str {
        self.text
    }

    #[must_use]
    pub const fn decode_map(self) -> Option<DecodeMap<'a>> {
        self.decoded
    }

    /// Borrow a checked decoded-relative piece without decoding again.
    /// Complete entities keep their authored spelling and map; partial entity
    /// boundaries remain errors. Identity-only pieces need no map allocation.
    pub fn slice_in(self, allocator: &'a Allocator, relative: Span) -> Result<Self, SourceError> {
        let text = self
            .text
            .get(relative.start as usize..relative.end as usize)
            .ok_or(SourceError::InvalidDecodedSpan)?;
        let span = self.authored_span(relative)?;
        if relative.start == 0 && relative.end as usize == self.text.len() {
            return Ok(self);
        }
        let decoded = self
            .decoded
            .map(|map| map.slice_in(allocator, relative))
            .transpose()?
            .flatten();
        Ok(Self {
            authored_root: self.authored_root,
            span,
            text,
            decoded,
        })
    }

    /// Exact decoded-relative to authored-file projection, suitable for edits.
    /// A partial entity expansion, including an interior point, is an explicit
    /// error; complete entities and their start/end boundary points are exact.
    pub fn authored_span(self, relative: Span) -> Result<Span, SourceError> {
        self.project(relative, false)
    }

    /// Diagnostic highlight projection: an endpoint inside an entity expands
    /// to cover its entire authored spelling. An interior zero-width point
    /// therefore becomes the full entity range. Boundary points stay points.
    /// This conservative projection must not be used to select rewrite bytes.
    pub fn authored_covering_span(self, relative: Span) -> Result<Span, SourceError> {
        self.project(relative, true)
    }

    fn project(self, relative: Span, covering: bool) -> Result<Span, SourceError> {
        self.text
            .get(relative.start as usize..relative.end as usize)
            .ok_or(SourceError::InvalidDecodedSpan)?;
        if let Some(map) = self.decoded {
            map.project(relative, covering)
        } else {
            Ok(Span::new(
                self.span
                    .start
                    .checked_add(relative.start)
                    .ok_or(SourceError::SourceTooLarge)?,
                self.span
                    .start
                    .checked_add(relative.end)
                    .ok_or(SourceError::SourceTooLarge)?,
            ))
        }
    }
}

fn checked_len(length: usize) -> Result<u32, SourceError> {
    u32::try_from(length).map_err(|_| SourceError::SourceTooLarge)
}

fn checked_span(start: usize, end: usize) -> Result<Span, SourceError> {
    Ok(Span::new(checked_len(start)?, checked_len(end)?))
}

fn checked_add(start: usize, length: usize) -> Result<usize, SourceError> {
    let end = start
        .checked_add(length)
        .ok_or(SourceError::SourceTooLarge)?;
    checked_len(end)?;
    Ok(end)
}

fn append_identity<'a>(
    output: &mut String,
    segments: &mut Vec<'a, DecodeSegment>,
    raw: &str,
    base: usize,
    start: usize,
    end: usize,
) -> Result<(), SourceError> {
    if start == end {
        return Ok(());
    }
    let literal = raw
        .get(start..end)
        .ok_or(SourceError::InvalidAuthoredSpan)?;
    let decoded_start = output.len();
    let decoded_end = checked_add(decoded_start, literal.len())?;
    output.push_str(literal);
    segments.push(DecodeSegment::new(
        checked_span(decoded_start, decoded_end)?,
        checked_span(checked_add(base, start)?, checked_add(base, end)?)?,
        DecodeSegmentKind::Identity,
    ));
    Ok(())
}

/// Prepare attribute value bytes, excluding their surrounding quotes.
///
/// Unknown references and reference-free values remain borrowed and allocate
/// nothing. Each valid reference is decoded from authored bytes once by the
/// ordinary native attribute decoder; decoded output is never decoded again.
/// This prepares source only, without selecting a language, shape or syntax tree.
pub fn prepare_attribute_value<'a>(
    allocator: &'a Allocator,
    authored_source: &'a str,
    span: Span,
) -> Result<EmbedSource<'a>, SourceError> {
    prepare_decoded_value(allocator, authored_source, span, EntityContext::Attribute)
}

fn prepare_decoded_value<'a>(
    allocator: &'a Allocator,
    authored_source: &'a str,
    span: Span,
    context: EntityContext,
) -> Result<EmbedSource<'a>, SourceError> {
    let plain = EmbedSource::authored(authored_source, span)?;
    let raw = plain.text();
    if !needs_decoding(raw.as_bytes()) {
        return Ok(plain);
    }
    let mut output = String::default();
    let mut segments = Vec::new_in(&allocator);
    let mut cursor = 0;
    for (at, _) in raw.match_indices('&') {
        if at < cursor {
            continue;
        }
        let Some((value, consumed)) = raw
            .as_bytes()
            .get(at..)
            .and_then(|bytes| decode_one(bytes, context))
        else {
            continue;
        };
        let end = checked_add(at, consumed)?;
        if consumed == 0 || raw.get(at..end).is_none() {
            return Err(SourceError::InvalidEntitySegment);
        }
        append_identity(
            &mut output,
            &mut segments,
            raw,
            span.start as usize,
            cursor,
            at,
        )?;
        let decoded_start = output.len();
        let mut width = Some(0usize);
        value.for_each(|scalar| {
            width = width.and_then(|n| n.checked_add(scalar.len_utf8()));
        });
        let decoded_end = checked_add(decoded_start, width.ok_or(SourceError::SourceTooLarge)?)?;
        value.for_each(|scalar| output.push(scalar));
        segments.push(DecodeSegment::new(
            checked_span(decoded_start, decoded_end)?,
            checked_span(
                checked_add(span.start as usize, at)?,
                checked_add(span.start as usize, end)?,
            )?,
            DecodeSegmentKind::Entity,
        ));
        cursor = end;
    }
    if segments.is_empty() {
        return Ok(plain);
    }
    append_identity(
        &mut output,
        &mut segments,
        raw,
        span.start as usize,
        cursor,
        raw.len(),
    )?;
    let text = allocator.alloc_str(output.as_str());
    let segments = segments.into_boxed_slice().into_arena_slice();
    let decoded = DecodeMap::checked(authored_source, span, text, segments)?;
    Ok(EmbedSource {
        authored_root: authored_source,
        span,
        text,
        decoded: Some(decoded),
    })
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod slice_tests;

#[cfg(test)]
mod origin_tests;
