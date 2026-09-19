//! The editor uses the same authored-node diagnostic policy as batch checking.

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};
use vize_canon::template_diagnostic_directives::{
    TemplateDiagnosticDirectives, UNUSED_EXPECT_ERROR_CODE, UNUSED_EXPECT_ERROR_MESSAGE,
};
use vize_s0::line_index::LineIndex;

pub(super) fn apply(source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut directives = TemplateDiagnosticDirectives::for_sfc(source);
    if directives.directives().is_empty() {
        return;
    }
    let lines = LineIndex::new(source);
    diagnostics.retain(|diagnostic| {
        let position = diagnostic.range.start;
        lines
            .line_col_to_offset(position.line, position.character)
            .is_none_or(|offset| !directives.suppresses(offset))
    });
    for range in directives.unused_expectations() {
        let (start_line, start_character) = lines.line_col(range.start);
        let (end_line, end_character) = lines.line_col(range.end);
        diagnostics.push(Diagnostic {
            range: Range::new(
                Position::new(start_line, start_character),
                Position::new(end_line, end_character),
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::Number(UNUSED_EXPECT_ERROR_CODE as i32)),
            source: Some(super::sources::TYPE_CHECKER.to_string()),
            message: UNUSED_EXPECT_ERROR_MESSAGE.to_string(),
            ..Diagnostic::default()
        });
    }
}
