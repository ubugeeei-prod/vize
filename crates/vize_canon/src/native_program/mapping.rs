//! Convert the existing bridge's exact LSP ranges through L4 source links.

use super::NativeProgramDiagnostic;
use crate::{LspPosition, LspRange};
use lsp_types::{Diagnostic, Range};
use vize_l0::{
    Span,
    line_index::{LineBreaks, utf16_len},
};
use vize_l4::targets::ts::{MappingError, ProgramProjection};

pub(super) fn observe(
    projection: &ProgramProjection<'_, '_>,
    backend: Diagnostic,
) -> NativeProgramDiagnostic {
    let span = map_range(projection, &backend.range);
    let original_range = span.ok().map(|span| {
        let source = projection.file().artifact().source();
        let position = |offset| {
            let (line, character) = LineBreaks::Lsp.offset_to_position(source, offset);
            LspPosition { line, character }
        };
        LspRange {
            start: position(span.start as usize),
            end: position(span.end as usize),
        }
    });
    NativeProgramDiagnostic {
        backend,
        span,
        original_range,
    }
}

fn map_range(projection: &ProgramProjection<'_, '_>, range: &Range) -> Result<Span, MappingError> {
    let text = projection.document().as_str();
    let offset = |position: &lsp_types::Position| {
        let byte = LineBreaks::Lsp
            .position_to_offset(text, position.line, position.character)
            .ok_or(MappingError::InvalidUtf16Boundary)?;
        let prefix = text.get(..byte).ok_or(MappingError::InvalidRange)?;
        u32::try_from(utf16_len(prefix)).map_err(|_| MappingError::InvalidUtf16Boundary)
    };
    let start = offset(&range.start)?;
    let end = offset(&range.end)?;
    let length = end.checked_sub(start).ok_or(MappingError::InvalidRange)?;
    projection.map_utf16(start, length)
}
