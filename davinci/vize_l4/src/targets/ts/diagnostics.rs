//! Share global checker positions while each projection owns its exact mappings.
use super::{MappingError, ProgramProjection, TsDocument, vue::VueProjection};
use vize_l0::{Span, line_index::Utf16Cursor};

/// Batch diagnostic mapping over a single borrowed generated document.
/// Each request uses the projection's original range mapping, including
/// generated text, nested links and decoded entities.
pub struct DiagnosticMapper<'p, M> {
    cursor: Utf16Cursor<'p>,
    map_span: M,
}

impl<'p, M: Fn(Span) -> Result<Span, MappingError>> DiagnosticMapper<'p, M> {
    #[inline]
    fn new(text: &'p str, map_span: M) -> Self {
        Self {
            cursor: Utf16Cursor::new(text),
            map_span,
        }
    }

    /// Map a checker range in global UTF-16 code units. Ranges may overlap
    /// or arrive in any order. Overflow and surrogate interiors are reported.
    #[inline]
    pub fn map_utf16(&mut self, start: u32, length: u32) -> Result<Span, MappingError> {
        let bytes = self
            .cursor
            .span(start, length)
            .ok_or(MappingError::InvalidUtf16Boundary)?;
        (self.map_span)(bytes)
    }
}

impl ProgramProjection<'_, '_> {
    /// Borrow this program's checker document for a batch of diagnostics.
    #[must_use]
    #[inline]
    pub fn diagnostic_mapper(
        &self,
    ) -> DiagnosticMapper<'_, impl Fn(Span) -> Result<Span, MappingError> + '_> {
        DiagnosticMapper::new(self.document().as_str(), |range| self.map_span(range))
    }
}

impl VueProjection<'_, '_> {
    /// Borrow this SFC's original setup and interpolation mappings.
    #[must_use]
    #[inline]
    pub fn diagnostic_mapper(
        &self,
    ) -> DiagnosticMapper<'_, impl Fn(Span) -> Result<Span, MappingError> + '_> {
        DiagnosticMapper::new(self.document().as_str(), |range| self.map_span(range))
    }
}

impl TsDocument {
    /// Borrow this JSX or TSX module's authored and generated link ranges.
    #[must_use]
    #[inline]
    pub fn diagnostic_mapper(
        &self,
    ) -> DiagnosticMapper<'_, impl Fn(Span) -> Result<Span, MappingError> + '_> {
        DiagnosticMapper::new(self.document().as_str(), |range| self.map_span(range))
    }
}
