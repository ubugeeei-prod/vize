//! Neutral coordinates for a retained AST whose parser used a private wrapper.

use vize_l0::Span;

/// One complete decoded/source correspondence; entity interiors are indivisible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsSegment {
    pub decoded: Span,
    pub authored: Span,
    pub entity: bool,
}

/// An invalid coordinate bridge is refused before an expression enters L2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsCoordinateError {
    SourceLimit,
    InvalidSpan,
    InvalidCoverage,
    InvalidIdentity,
    InvalidEntity,
    OutsideExpression,
}

/// Checked wrapper correction and decoded-to-authored projection.
///
/// This vocabulary is independent of L1 and HTML decoding. A producer transfers
/// the source preparation's correspondences; L2 never decodes or parses again.
#[derive(Debug, Clone, Copy)]
pub struct JsCoordinates<'a> {
    source: &'a str,
    authored: &'a str,
    span: Span,
    prefix: u32,
    segments: &'a [JsSegment],
}

impl<'a> JsCoordinates<'a> {
    /// Check complete UTF-8 coverage and literal segment identity once.
    /// An empty map denotes exact authored text, not unvalidated decoded text.
    pub fn checked(
        file: &'a str,
        source: &'a str,
        span: Span,
        prefix: u32,
        segments: &'a [JsSegment],
    ) -> Result<Self, JsCoordinateError> {
        u32::try_from(file.len()).map_err(|_| JsCoordinateError::SourceLimit)?;
        let length = u32::try_from(source.len()).map_err(|_| JsCoordinateError::SourceLimit)?;
        prefix
            .checked_add(length)
            .ok_or(JsCoordinateError::SourceLimit)?;
        let raw = file
            .get(span.start as usize..span.end as usize)
            .ok_or(JsCoordinateError::InvalidSpan)?;
        if segments.is_empty() {
            if source != raw {
                return Err(JsCoordinateError::InvalidIdentity);
            }
        } else {
            let (mut decoded, mut authored) = (0, span.start);
            for segment in segments {
                if segment.decoded.start != decoded
                    || segment.authored.start != authored
                    || segment.decoded.start >= segment.decoded.end
                    || segment.authored.start >= segment.authored.end
                    || segment.authored.end > span.end
                {
                    return Err(JsCoordinateError::InvalidCoverage);
                }
                let value = source
                    .get(segment.decoded.start as usize..segment.decoded.end as usize)
                    .ok_or(JsCoordinateError::InvalidSpan)?;
                let raw = file
                    .get(segment.authored.start as usize..segment.authored.end as usize)
                    .ok_or(JsCoordinateError::InvalidSpan)?;
                if segment.entity {
                    if !raw.starts_with('&') {
                        return Err(JsCoordinateError::InvalidEntity);
                    }
                } else if value != raw {
                    return Err(JsCoordinateError::InvalidIdentity);
                }
                (decoded, authored) = (segment.decoded.end, segment.authored.end);
            }
            if decoded != length || authored != span.end {
                return Err(JsCoordinateError::InvalidCoverage);
            }
        }
        Ok(Self {
            source,
            authored: raw,
            span,
            prefix,
            segments,
        })
    }

    pub(crate) fn matches(&self, source: &str, span: Span) -> bool {
        self.source == source && self.span == span
    }

    pub(crate) fn matches_authored_source(&self, file: &str) -> bool {
        file.get(self.span.start as usize..self.span.end as usize) == Some(self.authored)
    }

    pub(crate) fn decoded_span(&self, span: oxc_span::Span) -> Option<Span> {
        let span = Span::new(
            span.start.checked_sub(self.prefix)?,
            span.end.checked_sub(self.prefix)?,
        );
        self.source.get(span.start as usize..span.end as usize)?;
        Some(span)
    }

    pub(crate) fn authored_span(&self, span: Span) -> Option<Span> {
        self.source.get(span.start as usize..span.end as usize)?;
        if self.segments.is_empty() {
            return Some(Span::new(
                self.span.start.checked_add(span.start)?,
                self.span.start.checked_add(span.end)?,
            ));
        }
        Some(Span::new(self.offset(span.start)?, self.offset(span.end)?))
    }

    fn offset(&self, at: u32) -> Option<u32> {
        // Complete checked segments are sorted. Shared endpoints select the
        // prior segment and have the same authored offset in either neighbour.
        let index = self
            .segments
            .partition_point(|segment| segment.decoded.end < at);
        let segment = self.segments.get(index)?;
        if at < segment.decoded.start {
            return None;
        }
        if at == segment.decoded.start {
            return Some(segment.authored.start);
        }
        if at == segment.decoded.end {
            return Some(segment.authored.end);
        }
        (!segment.entity)
            .then(|| {
                segment
                    .authored
                    .start
                    .checked_add(at - segment.decoded.start)
            })
            .flatten()
    }
}

#[cfg(test)]
mod tests;
