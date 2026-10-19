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
mod tests {
    use super::{DecodeMap, DecodeSegment, DecodeSegmentKind as Kind, SourceError};
    use alloc::vec;
    use vize_l0::Span;

    fn segment(decoded: (u32, u32), authored: (u32, u32), kind: Kind) -> DecodeSegment {
        DecodeSegment::new(
            Span::new(decoded.0, decoded.1),
            Span::new(authored.0, authored.1),
            kind,
        )
    }

    #[test]
    fn coverage_rejects_gaps_overlaps_empty_and_incomplete_maps() {
        let invalid = [
            vec![],
            vec![segment((1, 2), (0, 2), Kind::Identity)],
            vec![segment((0, 2), (1, 2), Kind::Identity)],
            vec![segment((0, 1), (0, 1), Kind::Identity)],
            vec![segment((0, 0), (0, 1), Kind::Identity)],
            vec![segment((0, 1), (0, 0), Kind::Identity)],
            vec![
                segment((0, 1), (0, 1), Kind::Identity),
                segment((0, 2), (1, 2), Kind::Identity),
            ],
            vec![
                segment((0, 1), (0, 1), Kind::Identity),
                segment((1, 2), (0, 2), Kind::Identity),
            ],
            vec![segment((0, 2), (0, 3), Kind::Identity)],
            vec![
                segment((0, 2), (0, 2), Kind::Identity),
                segment((2, 1), (2, 2), Kind::Identity),
            ],
        ];
        for segments in invalid {
            assert_eq!(
                DecodeMap::checked("ab", Span::new(0, 2), "ab", &segments).unwrap_err(),
                SourceError::InvalidMapCoverage
            );
        }
        assert_eq!(
            DecodeMap::checked("", Span::new(0, 0), "", &[]).unwrap_err(),
            SourceError::InvalidMapCoverage
        );
    }

    #[test]
    fn coverage_checks_utf8_and_identity_bytes() {
        let segments = [segment((0, 2), (0, 2), Kind::Identity)];
        assert_eq!(
            DecodeMap::checked("ab", Span::new(0, 2), "ac", &segments).unwrap_err(),
            SourceError::InvalidIdentitySegment
        );
        let segments = [
            segment((0, 1), (0, 2), Kind::Identity),
            segment((1, 2), (2, 3), Kind::Identity),
        ];
        assert_eq!(
            DecodeMap::checked("éa", Span::new(0, 3), "é", &segments).unwrap_err(),
            SourceError::InvalidDecodedSpan
        );
        let segments = [
            segment((0, 2), (0, 1), Kind::Identity),
            segment((2, 3), (1, 3), Kind::Identity),
        ];
        assert_eq!(
            DecodeMap::checked("éa", Span::new(0, 3), "éa", &segments).unwrap_err(),
            SourceError::InvalidAuthoredSpan
        );
        assert_eq!(
            DecodeMap::checked("é", Span::new(1, 2), "", &[]).unwrap_err(),
            SourceError::InvalidAuthoredSpan
        );
        let segments = [segment((0, 1), (0, 1), Kind::Entity)];
        assert_eq!(
            DecodeMap::checked("a", Span::new(0, 1), "a", &segments).unwrap_err(),
            SourceError::InvalidEntitySegment
        );
    }

    #[test]
    fn exact_and_covering_projections_are_distinct_at_entity_interiors() {
        let segments = [segment((0, 2), (2, 9), Kind::Entity)];
        let map = DecodeMap::checked("xx&fjlig;z", Span::new(2, 9), "fj", &segments).unwrap();
        for span in [Span::new(0, 1), Span::new(1, 2), Span::new(1, 1)] {
            assert_eq!(
                map.project(span, false),
                Err(SourceError::PartialEntityBoundary)
            );
            assert_eq!(map.project(span, true), Ok(Span::new(2, 9)));
        }
        for (decoded, authored) in [(0, 2), (2, 9)] {
            for covering in [false, true] {
                assert_eq!(
                    map.project(Span::new(decoded, decoded), covering),
                    Ok(Span::new(authored, authored))
                );
            }
        }
        assert_eq!(map.project(Span::new(0, 2), false), Ok(Span::new(2, 9)));
    }

    #[test]
    fn equal_length_entity_is_never_treated_as_identity() {
        let segments = [segment((0, 5), (0, 5), Kind::Entity)];
        let map =
            DecodeMap::checked("&acE;", Span::new(0, 5), "\u{223e}\u{0333}", &segments).unwrap();
        assert_eq!(
            map.project(Span::new(0, 3), false),
            Err(SourceError::PartialEntityBoundary)
        );
        assert_eq!(map.project(Span::new(3, 3), true), Ok(Span::new(0, 5)));
    }

    #[test]
    fn adjacent_entity_and_identity_endpoints_are_deterministic() {
        let segments = [
            segment((0, 1), (1, 2), Kind::Identity),
            segment((1, 3), (2, 9), Kind::Entity),
            segment((3, 4), (9, 14), Kind::Entity),
            segment((4, 6), (14, 16), Kind::Identity),
        ];
        let map =
            DecodeMap::checked("xa&fjlig;&amp;éz", Span::new(1, 16), "afj&é", &segments).unwrap();
        for (decoded, authored) in [(0, 1), (1, 2), (3, 9), (4, 14), (6, 16)] {
            for covering in [false, true] {
                assert_eq!(
                    map.project(Span::new(decoded, decoded), covering),
                    Ok(Span::new(authored, authored))
                );
            }
        }
        assert_eq!(map.project(Span::new(2, 6), true), Ok(Span::new(2, 16)));
        assert_eq!(map.project(Span::new(0, 2), true), Ok(Span::new(1, 9)));
    }
}
