use super::{
    DiagnosticMapper, parse_diagnostic_code, parse_severity, should_skip_diagnostic,
    should_skip_original_diagnostic,
};
use crate::batch::Diagnostic;
use crate::corsa_client::LspDiagnostic;
use std::path::Path;

impl DiagnosticMapper<'_> {
    pub(super) fn map_lsp_diagnostic(
        &mut self,
        virtual_path: &Path,
        diagnostic: LspDiagnostic,
    ) -> Option<Diagnostic> {
        let code = parse_diagnostic_code(diagnostic.code.as_ref());
        if should_skip_diagnostic(code, &diagnostic.message) {
            return None;
        }
        if code == Some(6133) && !self.preserve_unused_diagnostics {
            return None;
        }
        if code == Some(2322)
            && self.is_keyof_indexed_assignment(
                virtual_path,
                diagnostic.range.start.line,
                diagnostic.range.start.character,
            )
        {
            return None;
        }

        let original = self.map_diagnostic_position_to_original(virtual_path, &diagnostic, code);
        if original
            .as_ref()
            .is_some_and(|original| should_skip_original_diagnostic(code, original))
        {
            return None;
        }

        if let Some(original) = original {
            return Some(Diagnostic {
                message: self.authored_message(&original, diagnostic.message),
                line: original.line,
                column: original.column,
                file: original.path,
                code,
                severity: if self.is_unreachable_pattern(
                    virtual_path,
                    diagnostic.range.start.line,
                    diagnostic.range.start.character,
                    code,
                ) {
                    2
                } else {
                    parse_severity(diagnostic.severity)
                },
                block_type: original.block_type,
            });
        }

        None
    }
}
