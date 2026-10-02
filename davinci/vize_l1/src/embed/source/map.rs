use vize_l0::{Allocator, Span, Vec};

use super::{SourceError, checked_len};

/// Equal lengths do not imply identity: `&acE;` expands to five UTF-8 bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeSegmentKind {
    Identity,
    Entity,
}

/// One contiguous correspondence. Entity spelling is an indivisible atom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodeSegment {
    decoded: Span,
    authored: Span,
    kind: DecodeSegmentKind,
}

impl DecodeSegment {
    pub(super) const fn new(decoded: Span, authored: Span, kind: DecodeSegmentKind) -> Self {
        Self {
            decoded,
            authored,
            kind,
        }
    }
    #[must_use]
    pub const fn decoded(self) -> Span {
        self.decoded
    }
    #[must_use]
    pub const fn authored(self) -> Span {
        self.authored
    }
    #[must_use]
    pub const fn kind(self) -> DecodeSegmentKind {
        self.kind
    }
}

/// Complete, ordered decoded/authored coverage produced by native preparation.
/// Arbitrary segment sets cannot be supplied by external consumers.
#[derive(Debug, Clone, Copy)]
pub struct DecodeMap<'a> {
    segments: &'a [DecodeSegment],
}

impl<'a> DecodeMap<'a> {
    #[must_use]
    pub const fn segments(self) -> &'a [DecodeSegment] {
        self.segments
    }

    pub(super) fn checked(
        source: &str,
        span: Span,
        text: &str,
        segments: &'a [DecodeSegment],
    ) -> Result<Self, SourceError> {
        checked_len(source.len())?;
        let decoded_len = checked_len(text.len())?;
        source
            .get(span.start as usize..span.end as usize)
            .ok_or(SourceError::InvalidAuthoredSpan)?;
        if segments.is_empty() {
            return Err(SourceError::InvalidMapCoverage);
        }
        let (mut decoded, mut authored) = (0, span.start);
        for segment in segments {
            if segment.decoded.start != decoded
                || segment.authored.start != authored
                || segment.decoded.start >= segment.decoded.end
                || segment.authored.start >= segment.authored.end
                || segment.authored.end > span.end
            {
                return Err(SourceError::InvalidMapCoverage);
            }
            let value = text
                .get(segment.decoded.start as usize..segment.decoded.end as usize)
                .ok_or(SourceError::InvalidDecodedSpan)?;
            let raw = source
                .get(segment.authored.start as usize..segment.authored.end as usize)
                .ok_or(SourceError::InvalidAuthoredSpan)?;
            match segment.kind {
                DecodeSegmentKind::Identity if value != raw => {
                    return Err(SourceError::InvalidIdentitySegment);
                }
                DecodeSegmentKind::Entity if !raw.starts_with('&') => {
                    return Err(SourceError::InvalidEntitySegment);
                }
                _ => {}
            }
            (decoded, authored) = (segment.decoded.end, segment.authored.end);
        }
        if decoded != decoded_len || authored != span.end {
            return Err(SourceError::InvalidMapCoverage);
        }
        Ok(Self { segments })
    }

    pub(super) fn project(self, span: Span, covering: bool) -> Result<Span, SourceError> {
        Ok(Span::new(
            self.offset(span.start, covering, false)?,
            self.offset(span.end, covering, true)?,
        ))
    }

    /// Derive a private map from already checked parent segments. Boundary
    /// projection rejects cuts inside entities; identity segments can shorten.
    /// No authored bytes are decoded again, and no unchecked public constructor
    /// is introduced. Returned segments cover the new relative text completely.
    pub(super) fn slice_in(
        self,
        allocator: &'a Allocator,
        span: Span,
    ) -> Result<Option<Self>, SourceError> {
        self.project(span, false)?;
        let has_entity = self.segments.iter().any(|segment| {
            segment.kind == DecodeSegmentKind::Entity
                && segment.decoded.start < span.end
                && span.start < segment.decoded.end
        });
        if !has_entity {
            return Ok(None);
        }
        let mut segments = Vec::new_in(&allocator);
        for segment in self.segments {
            let start = segment.decoded.start.max(span.start);
            let end = segment.decoded.end.min(span.end);
            if start >= end {
                continue;
            }
            let authored = match segment.kind {
                DecodeSegmentKind::Identity => Span::new(
                    segment
                        .authored
                        .start
                        .checked_add(start - segment.decoded.start)
                        .ok_or(SourceError::SourceTooLarge)?,
                    segment
                        .authored
                        .start
                        .checked_add(end - segment.decoded.start)
                        .ok_or(SourceError::SourceTooLarge)?,
                ),
                DecodeSegmentKind::Entity => segment.authored,
            };
            segments.push(DecodeSegment::new(
                Span::new(start - span.start, end - span.start),
                authored,
                segment.kind,
            ));
        }
        Ok(Some(Self {
            segments: segments.into_boxed_slice().into_arena_slice(),
        }))
    }

    fn offset(self, at: u32, covering: bool, end: bool) -> Result<u32, SourceError> {
        for segment in self.segments {
            if at < segment.decoded.start || at > segment.decoded.end {
                continue;
            }
            if at == segment.decoded.start {
                return Ok(segment.authored.start);
            }
            if at == segment.decoded.end {
                return Ok(segment.authored.end);
            }
            return match segment.kind {
                DecodeSegmentKind::Identity => segment
                    .authored
                    .start
                    .checked_add(at - segment.decoded.start)
                    .ok_or(SourceError::SourceTooLarge),
                DecodeSegmentKind::Entity if covering => Ok(if end {
                    segment.authored.end
                } else {
                    segment.authored.start
                }),
                DecodeSegmentKind::Entity => Err(SourceError::PartialEntityBoundary),
            };
        }
        Err(SourceError::InvalidMapCoverage)
    }
}

#[cfg(test)]
mod tests;
