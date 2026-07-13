//! Script-block diagnostics projected from the persistent Module product.

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Url};
use vize_atelier_sfc::SfcDescriptor;

use crate::server::ServerState;

use super::super::{DiagnosticService, LineIndex, sources};

impl DiagnosticService {
    pub(in crate::ide::diagnostics) fn collect_script_diagnostics(
        state: &ServerState,
        uri: &Url,
        content: &str,
        descriptor: &SfcDescriptor<'_>,
        line_index: &LineIndex<'_>,
    ) -> Vec<Diagnostic> {
        let Some(modules) = state.sfc_modules(uri) else {
            return Vec::new();
        };

        modules
            .modules
            .iter()
            .filter(|module| module_language_is_supported(descriptor, &module.name))
            .flat_map(|module| &module.diagnostics)
            .map(|error| {
                let start = error.span.start as usize;
                let end = error.span.end as usize;
                let (start_line, start_col) = line_index.line_col(start.min(content.len()));
                let (end_line, end_col) = line_index.line_col(end.min(content.len()));
                Diagnostic {
                    range: Range {
                        start: Position {
                            line: start_line,
                            character: start_col,
                        },
                        end: Position {
                            line: end_line,
                            character: end_col,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("script-parse-error".to_string())),
                    source: Some(sources::SCRIPT_PARSER.to_string()),
                    message: format!("Script parse error: {}", error.message),
                    ..Default::default()
                }
            })
            .collect()
    }
}

fn module_language_is_supported(descriptor: &SfcDescriptor<'_>, name: &str) -> bool {
    let lang = if name.ends_with("#script-setup") {
        descriptor
            .script_setup
            .as_ref()
            .and_then(|script| script.lang.as_deref())
    } else {
        descriptor
            .script
            .as_ref()
            .and_then(|script| script.lang.as_deref())
    };
    lang.is_none_or(|lang| {
        matches!(
            lang.trim().to_ascii_lowercase().as_str(),
            "" | "js" | "javascript" | "jsx" | "ts" | "typescript" | "tsx"
        )
    })
}
