use vize_l0::Span;

use super::{EmbedSource, SourceError};

#[derive(Debug, Clone, Copy)]
pub(super) struct Coordinates<'a> {
    pub source: EmbedSource<'a>,
    pub prefix: u32,
}

impl Coordinates<'_> {
    pub fn decoded_span(self, span: oxc_span::Span) -> Result<Span, SourceError> {
        let start = span
            .start
            .checked_sub(self.prefix)
            .ok_or(SourceError::InvalidDecodedSpan)?;
        let end = span
            .end
            .checked_sub(self.prefix)
            .ok_or(SourceError::InvalidDecodedSpan)?;
        self.source
            .text()
            .get(start as usize..end as usize)
            .ok_or(SourceError::InvalidDecodedSpan)?;
        Ok(Span::new(start, end))
    }

    /// Diagnostic labels can include generated delimiters. Intersect them with
    /// real source, anchoring wrapper-only labels to the start/end source point.
    /// If a label splits UTF-8, cover its whole scalar instead of inventing an
    /// interior authored byte. This operation must never select rewrite bytes.
    pub fn diagnostic_span(self, start: u32, end: u32) -> Result<Span, SourceError> {
        if start > end {
            return Err(SourceError::InvalidDecodedSpan);
        }
        let text = self.source.text();
        let mut start = (start.saturating_sub(self.prefix) as usize).min(text.len());
        let mut end = (end.saturating_sub(self.prefix) as usize).min(text.len());
        while !text.is_char_boundary(start) {
            start -= 1;
        }
        while !text.is_char_boundary(end) {
            end += 1;
        }
        let span = Span::new(start as u32, end as u32);
        text.get(start..end)
            .ok_or(SourceError::InvalidDecodedSpan)?;
        Ok(span)
    }
}
