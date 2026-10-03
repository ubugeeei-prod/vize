//! Exact range links are the checker mapping; generated punctuation is unlinked.

use super::ProgramProjection;
use vize_l0::{Span, line_index::utf16_offset};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingError {
    Unrecorded,
    InvalidRange,
    InvalidUtf16Boundary,
    GeneratedOnly,
    CrossesBoundary,
}

impl ProgramProjection<'_, '_> {
    /// Map complete UTF-8 byte ranges, including a point at the authored end.
    pub fn map_span(&self, generated: Span) -> Result<Span, MappingError> {
        if !self.document().is_recording() {
            return Err(MappingError::Unrecorded);
        }
        if generated.start > generated.end
            || self
                .document()
                .as_str()
                .get(generated.start as usize..generated.end as usize)
                .is_none()
        {
            return Err(MappingError::InvalidRange);
        }
        let end = self.unit().span.end;
        if generated.start > end || (generated.start == end && generated.end > end) {
            return Err(MappingError::GeneratedOnly);
        }
        if generated.end > end {
            return Err(MappingError::CrossesBoundary);
        }
        Ok(generated)
    }

    /// Convert actual TypeScript global UTF-16 offsets before exact range mapping.
    /// Mid-surrogate positions, overflow and generated-only ranges never clamp.
    pub fn map_utf16(&self, start: u32, length: u32) -> Result<Span, MappingError> {
        let end = start
            .checked_add(length)
            .ok_or(MappingError::InvalidUtf16Boundary)?;
        let text = self.document().as_str();
        let byte = |at| {
            utf16_offset(text, at)
                .and_then(|offset| u32::try_from(offset).ok())
                .ok_or(MappingError::InvalidUtf16Boundary)
        };
        self.map_span(Span::new(byte(start)?, byte(end)?))
    }
}
